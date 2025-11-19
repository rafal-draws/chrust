from contextlib import asynccontextmanager
import os
from fastapi.responses import HTMLResponse
import redis

from typing import Annotated
from fastapi import FastAPI, HTTPException, Header, Response
from dotenv import load_dotenv
from apscheduler.schedulers.background import BackgroundScheduler
from apscheduler.triggers.cron import CronTrigger
import asyncio
from concurrent.futures import ThreadPoolExecutor

from server_utils import artifacts_gen




load_dotenv()
r = redis.Redis(host="redis", port=6379, decode_responses=True)

server_data = os.path.join("/server_data")
metadata = os.path.join("/metadata")

try:
    os.mkdir(os.path.join(server_data, "uploads"))
except FileExistsError as e:
    print("Dir exists:", e)

os.system("sh download_utils.sh")



@asynccontextmanager
async def lifespan(app: FastAPI):
    scheduler.start()
    print("Scheduler started.")
    yield
    scheduler.shutdown()
    print("Scheduler shut down.")


app = FastAPI(title="backend-etl", lifespan=lifespan)


from fastapi.middleware.cors import CORSMiddleware

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # before production ["http://localhost:3000"]
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


def cleanup_files():
    count = 0
    dirs = [os.path.join(server_data, folder) for folder in os.listdir(server_data)]
    for dir in dirs:
        files = [os.path.join(dir, file) for file in os.listdir(dir)]
        for file in files:
            try:
                os.remove(file)
            except IsADirectoryError:
                try:
                    os.removedirs(file)
                except Exception as e:
                    print(f"Could not delete dir {file}: {e}")
            except Exception as e:
                print(f"Could not delete file {file}: {e}")
            finally:
                count += 1
    print(f"[CRON JOB] Cleaned {count} files/folders")



scheduler = BackgroundScheduler()
scheduler.add_job(cleanup_files, CronTrigger(minute=0, hour='*/4'))  



@app.get("/")
def read_root():
    return {
        "hi"
    }


@app.get("/status/{track_id}", response_class=HTMLResponse)
def check_status(track_id: str):

    status_progress = r.get(track_id)

    if status_progress == None:
        return "Transformation not started!"
    
    # Handle both old format ("100%") and new format ("100")
    if status_progress.startswith("ERROR"):
        return f"<div style='color: red;'>{status_progress}</div>"
    
    # Convert to int to compare
    try:
        progress_int = int(status_progress.replace("%", ""))
    except ValueError:
        return f"<div style='color: orange;'>Unknown status: {status_progress}</div>"
    
    if progress_int < 100:
        return f"{progress_int}%"
    else:
        html = f'<div><a href="/track/{track_id}"><button>Explore the results!</button></a></div>'
        return html

@app.get("/transform/{song_id}")
async def transform_signal_and_populate_server_data(song_id: str):
    """
    Async version that parallelizes all 9 independent transformations after preprocessing.
    Sequential phase: load audio → extract → split frames (0-10%)
    Parallel phase: all 9 transformations run concurrently (10-100%, ~10% each)
    """
    print(f"song_id: {song_id}")
    print(f"server_data: {server_data}")

    hop_size = 2205
    signal_length = 30
    generate_video = True
    generate_features = True

    try:
        loop = asyncio.get_event_loop()
        
        # Initialize progress tracking
        r.set(song_id, "0")
        
        # Phase 1: Sequential preprocessing (must happen in order)
        song = await loop.run_in_executor(None, artifacts_gen.validate_audio_files, server_data, song_id)
        r.set(song_id, "2")

        print(f"song[0]: {song[0]}")
        y = await loop.run_in_executor(None, artifacts_gen.infer_signals, os.path.join(server_data, "uploads", song[0]))
        r.set(song_id, "4")

        y_30 = await loop.run_in_executor(None, artifacts_gen.extract_y_middle, y, signal_length)
        r.set(song_id, "6")

        sample_location = await loop.run_in_executor(None, artifacts_gen.generate_audio_from_frames, song_id, y_30, 22050, server_data)
        r.set(song_id, "8")
        
        y_frames = await loop.run_in_executor(None, artifacts_gen.split_to_frames, y_30, 22050, hop_size)
        r.set(song_id, "10")

        # Phase 2: Parallel transformations (all run concurrently)
        # Each task contributes ~10% to progress (10% → 100%)
        async def process_ft():
            try:
                frames_ft = await loop.run_in_executor(None, artifacts_gen.transform_to_ft, y_frames, metadata, True)
                current = int(r.get(song_id) or 10)
                r.set(song_id, str(min(current + 3, 100)))
                
                if generate_video:
                    await loop.run_in_executor(None, artifacts_gen.generate_ft_graphs, frames_ft, server_data, song_id)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 3, 100)))
                    await loop.run_in_executor(None, artifacts_gen.generate_video, server_data, song_id, "ft", sample_location)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 4, 100)))
                
                if generate_features:
                    await loop.run_in_executor(None, artifacts_gen.save_feature_to_server_data, "ft", server_data, song_id, frames_ft)
                
                del frames_ft
                return "ft_complete"
            except Exception as e:
                print(f"Error in process_ft: {e}")
                raise

        async def process_spectr():
            try:
                spectr_normalized = await loop.run_in_executor(None, artifacts_gen.transform_to_spectr, y_frames, metadata, True)
                current = int(r.get(song_id) or 10)
                r.set(song_id, str(min(current + 3, 100)))

                if generate_video:
                    await loop.run_in_executor(None, artifacts_gen.generate_spectrogram_graphs, spectr_normalized, server_data, song_id)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 3, 100)))
                    await loop.run_in_executor(None, artifacts_gen.generate_video, server_data, song_id, "spectr", sample_location)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 4, 100)))
                
                if generate_features:
                    await loop.run_in_executor(None, artifacts_gen.save_feature_to_server_data, "spectr", server_data, song_id, spectr_normalized)
                
                del spectr_normalized
                return "spectr_complete"
            except Exception as e:
                print(f"Error in process_spectr: {e}")
                raise

        async def process_mel_spectr():
            try:
                mel_spectr_normalized = await loop.run_in_executor(None, artifacts_gen.transform_to_mel_spectr, y_frames, metadata, True)
                current = int(r.get(song_id) or 10)
                r.set(song_id, str(min(current + 3, 100)))

                if generate_video:
                    await loop.run_in_executor(None, artifacts_gen.generate_mel_spectrogram_graphs, mel_spectr_normalized, server_data, song_id)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 3, 100)))
                    await loop.run_in_executor(None, artifacts_gen.generate_video, server_data, song_id, "mel_spectr", sample_location)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 4, 100)))

                if generate_features:
                    await loop.run_in_executor(None, artifacts_gen.save_feature_to_server_data, "mel_spectr", server_data, song_id, mel_spectr_normalized)

                del mel_spectr_normalized
                return "mel_spectr_complete"
            except Exception as e:
                print(f"Error in process_mel_spectr: {e}")
                raise

        async def process_power_spectr():
            try:
                power_spectr_normalized = await loop.run_in_executor(None, artifacts_gen.transform_to_power_spectr, y_frames, metadata, True)
                current = int(r.get(song_id) or 10)
                r.set(song_id, str(min(current + 3, 100)))

                if generate_video:
                    await loop.run_in_executor(None, artifacts_gen.generate_power_spectrogram_graphs, power_spectr_normalized, server_data, song_id)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 3, 100)))
                    await loop.run_in_executor(None, artifacts_gen.generate_video, server_data, song_id, "power_spectr", sample_location)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 4, 100)))

                if generate_features:
                    await loop.run_in_executor(None, artifacts_gen.save_feature_to_server_data, "power_spectr", server_data, song_id, power_spectr_normalized)
                
                del power_spectr_normalized
                return "power_spectr_complete"
            except Exception as e:
                print(f"Error in process_power_spectr: {e}")
                raise

        async def process_mfcc():
            try:
                mfcc_normalized = await loop.run_in_executor(None, artifacts_gen.transform_to_mfcc, y_frames, metadata, True)
                current = int(r.get(song_id) or 10)
                r.set(song_id, str(min(current + 3, 100)))

                if generate_video:
                    await loop.run_in_executor(None, artifacts_gen.generate_mfcc_graphs, mfcc_normalized, server_data, song_id)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 3, 100)))
                    await loop.run_in_executor(None, artifacts_gen.generate_video, server_data, song_id, "mfcc", sample_location)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 4, 100)))

                if generate_features:
                    await loop.run_in_executor(None, artifacts_gen.save_feature_to_server_data, "mfcc", server_data, song_id, mfcc_normalized)

                del mfcc_normalized
                return "mfcc_complete"
            except Exception as e:
                print(f"Error in process_mfcc: {e}")
                raise

        async def process_chroma_stft():
            try:
                normalized_chroma_stft = await loop.run_in_executor(None, artifacts_gen.transform_to_chroma, y_frames, metadata, "stft", True)
                current = int(r.get(song_id) or 10)
                r.set(song_id, str(min(current + 3, 100)))

                if generate_video:
                    await loop.run_in_executor(None, artifacts_gen.generate_chroma_graphs, normalized_chroma_stft, "stft", server_data, song_id)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 3, 100)))
                    await loop.run_in_executor(None, artifacts_gen.generate_video, server_data, song_id, "stft", sample_location)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 4, 100)))

                if generate_features:
                    await loop.run_in_executor(None, artifacts_gen.save_feature_to_server_data, "chroma_stft", server_data, song_id, normalized_chroma_stft)
                
                del normalized_chroma_stft
                return "chroma_stft_complete"
            except Exception as e:
                print(f"Error in process_chroma_stft: {e}")
                raise

        async def process_chroma_cens():
            try:
                normalized_chroma_cens = await loop.run_in_executor(None, artifacts_gen.transform_to_chroma, y_frames, metadata, "cens", True)
                current = int(r.get(song_id) or 10)
                r.set(song_id, str(min(current + 3, 100)))

                if generate_video:
                    await loop.run_in_executor(None, artifacts_gen.generate_chroma_graphs, normalized_chroma_cens, "cens", server_data, song_id)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 3, 100)))
                    await loop.run_in_executor(None, artifacts_gen.generate_video, server_data, song_id, "cens", sample_location)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 4, 100)))

                if generate_features:
                    await loop.run_in_executor(None, artifacts_gen.save_feature_to_server_data, "chroma_cens", server_data, song_id, normalized_chroma_cens)
                
                del normalized_chroma_cens
                return "chroma_cens_complete"
            except Exception as e:
                print(f"Error in process_chroma_cens: {e}")
                raise

        async def process_chroma_cqt():
            try:
                normalized_chroma_cqt = await loop.run_in_executor(None, artifacts_gen.transform_to_chroma, y_frames, metadata, "cqt", True)
                current = int(r.get(song_id) or 10)
                r.set(song_id, str(min(current + 3, 100)))

                if generate_video:
                    await loop.run_in_executor(None, artifacts_gen.generate_chroma_graphs, normalized_chroma_cqt, "cqt", server_data, song_id)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 3, 100)))
                    await loop.run_in_executor(None, artifacts_gen.generate_video, server_data, song_id, "cqt", sample_location)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 4, 100)))

                if generate_features:
                    await loop.run_in_executor(None, artifacts_gen.save_feature_to_server_data, "chroma_cqt", server_data, song_id, normalized_chroma_cqt)

                del normalized_chroma_cqt
                return "chroma_cqt_complete"
            except Exception as e:
                print(f"Error in process_chroma_cqt: {e}")
                raise

        async def process_tonnetz():
            try:
                normalized_tonnetz = await loop.run_in_executor(None, artifacts_gen.transform_to_tonnetz, y_frames, metadata, True)
                current = int(r.get(song_id) or 10)
                r.set(song_id, str(min(current + 3, 100)))
                
                if generate_video:
                    await loop.run_in_executor(None, artifacts_gen.generate_tonnetz_graphs, normalized_tonnetz, server_data, song_id)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 3, 100)))
                    await loop.run_in_executor(None, artifacts_gen.generate_video, server_data, song_id, "tonnetz", sample_location)
                    current = int(r.get(song_id) or 10)
                    r.set(song_id, str(min(current + 4, 100)))
                
                if generate_features:
                    await loop.run_in_executor(None, artifacts_gen.save_feature_to_server_data, "tonnetz", server_data, song_id, normalized_tonnetz)
                
                del normalized_tonnetz
                return "tonnetz_complete"
            except Exception as e:
                print(f"Error in process_tonnetz: {e}")
                raise

        # Execute all 9 transformations in parallel using asyncio.gather
        print(f"Starting parallel processing of {song_id}")
        results = await asyncio.gather(
            process_ft(),
            process_spectr(),
            process_mel_spectr(),
            process_power_spectr(),
            process_mfcc(),
            process_chroma_stft(),
            process_chroma_cens(),
            process_chroma_cqt(),
            process_tonnetz(),
            return_exceptions=True
        )

        # Check for any errors
        errors = [r for r in results if isinstance(r, Exception)]
        if errors:
            print(f"Errors during parallel processing: {errors}")
            r.set(song_id, f"ERROR: {errors[0]}")
            raise HTTPException(status_code=500, detail=f"Processing failed: {errors[0]}")

        print(f"All transformations completed successfully: {results}")
        
        # Ensure final status is 100%
        r.set(song_id, "100")

        return {
            "status": "success",
            "message": "Signal transformed and server data populated successfully.",
            "song_id": song_id,
            "server_data": server_data
        }

    except Exception as e:
        print(f"Error processing {song_id}: {e}")
        r.set(song_id, f"ERROR: {str(e)}")
        raise HTTPException(status_code=500, detail=str(e))
