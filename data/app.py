from contextlib import asynccontextmanager
import os
from fastapi.responses import HTMLResponse
import redis

from typing import Annotated
from fastapi import FastAPI, HTTPException, Header, Response
from dotenv import load_dotenv
from apscheduler.schedulers.background import BackgroundScheduler
from apscheduler.triggers.cron import CronTrigger

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
    if not status_progress == "100%":
        return status_progress
    else:
        html = f'<div><a href="/track/{track_id}"><button>Explore the results!</button></a></div>'
        return html

@app.get("/transform/{song_id}")
def transform_signal_and_populate_server_data(song_id: str):

    print(f"song_id: {song_id}")
    print(f"server_data: {server_data}")

    hop_size = 2205
    signal_length = 30

    generate_video = True
    generate_features = True

    
    song    = artifacts_gen.validate_audio_files(server_data, song_id)
    r.set(song_id, "2%")

    print(f"song[0]: {song[0]}")
    y       = artifacts_gen.infer_signals(os.path.join(server_data, "uploads", song[0]))
    r.set(song_id, "4%")


    y_30    = artifacts_gen.extract_y_middle(y, signal_length)
    r.set(song_id, "6%")

    sample_location = artifacts_gen.generate_audio_from_frames(song_id, y_30, 22050, server_data)
    r.set(song_id, "8%")
    
    y = artifacts_gen.split_to_frames(y_30, frame_length=22050, hop_length=hop_size)
    r.set(song_id, "10%")

    frames_ft = artifacts_gen.transform_to_ft(y, metadata, True)
    r.set(song_id, "12%")

    if generate_video:
        artifacts_gen.generate_ft_graphs(frames_ft, server_data, song_id)
        r.set(song_id, "14%")
        artifacts_gen.generate_video(server_data, song_id, "ft", sample_location)
        r.set(song_id, "16%")
    if generate_features:
        artifacts_gen.save_feature_to_server_data("ft", server_data, song_id, frames_ft) 
        r.set(song_id, "18%")
    
    # saving the planet
    del frames_ft
    
    # op7
    spectr_normalized = artifacts_gen.transform_to_spectr(y, metadata, True)
    r.set(song_id, "20%")

    if generate_video:
        artifacts_gen.generate_spectrogram_graphs(spectr_normalized, server_data ,song_id)
        r.set(song_id, "22%")
        artifacts_gen.generate_video(server_data, song_id, "spectr", sample_location)
        r.set(song_id, "24%")   
    
    if generate_features:
        artifacts_gen.save_feature_to_server_data("spectr", server_data, song_id, spectr_normalized)
        r.set(song_id, "26%")
    del spectr_normalized

    # op8
    mel_spectr_normalized = artifacts_gen.transform_to_mel_spectr(y, metadata, True)
    r.set(song_id, "28%")

    if generate_video:

        artifacts_gen.generate_mel_spectrogram_graphs(mel_spectr_normalized, server_data, song_id)
        r.set(song_id, "30%")
        
        artifacts_gen.generate_video(server_data, song_id, "mel_spectr", sample_location)
        r.set(song_id, "32%")

    if generate_features:
        artifacts_gen.save_feature_to_server_data("mel_spectr", server_data, song_id, mel_spectr_normalized)
        r.set(song_id, "34%")

    del mel_spectr_normalized
    
    # op9
    power_spectr_normalized = artifacts_gen.transform_to_power_spectr(y, metadata, True)
    r.set(song_id, "36%")

    if generate_features:
                
        artifacts_gen.generate_power_spectrogram_graphs(power_spectr_normalized, server_data, song_id)
        r.set(song_id, "38%")
        artifacts_gen.generate_video(server_data, song_id, "power_spectr", sample_location)
        r.set(song_id, "40%")

    if generate_features:

        artifacts_gen.save_feature_to_server_data("power_spectr", server_data, song_id, power_spectr_normalized)
        r.set(song_id, "42%")
    del power_spectr_normalized

    
    # op 10
    mfcc_normalized = artifacts_gen.transform_to_mfcc(y, metadata, True)
    r.set(song_id, "44%")

    if generate_video:

        artifacts_gen.generate_mfcc_graphs(mfcc_normalized, server_data, song_id)
        r.set(song_id, "46%")
        artifacts_gen.generate_video(server_data, song_id, "mfcc", sample_location)
        r.set(song_id, "48%")

    if generate_features:
        artifacts_gen.save_feature_to_server_data("mfcc", server_data, song_id, mfcc_normalized)
        r.set(song_id, "50%")

    del mfcc_normalized

    # op 11
    normalized_chroma_stft = artifacts_gen.transform_to_chroma(y, metadata,  "stft", True)
    r.set(song_id, "52%")

    if generate_video:

        artifacts_gen.generate_chroma_graphs(normalized_chroma_stft, "stft", server_data, song_id)
        r.set(song_id, "54%")
        artifacts_gen.generate_video(server_data, song_id, "stft", sample_location)
        r.set(song_id, "56%")

    if generate_features:
        artifacts_gen.save_feature_to_server_data("chroma_stft", server_data, song_id, normalized_chroma_stft)
        r.set(song_id, "58%")
    del normalized_chroma_stft
    
    # op 12
    normalized_chroma_cens = artifacts_gen.transform_to_chroma(y, metadata,  "cens", True)

    if generate_video:
        r.set(song_id, "60%")

        artifacts_gen.generate_chroma_graphs(normalized_chroma_cens, "cens", server_data, song_id)
        r.set(song_id, "62%")
        artifacts_gen.generate_video(server_data, song_id, "cens", sample_location)
        r.set(song_id, "64%")

    if generate_features:
        artifacts_gen.save_feature_to_server_data("chroma_cens", server_data, song_id, normalized_chroma_cens)
        r.set(song_id, "66%")
    del normalized_chroma_cens
    
    # op 13
    normalized_chroma_cqt = artifacts_gen.transform_to_chroma(y, metadata,  "cqt", True)
    r.set(song_id, "68%")

    if generate_video:
        artifacts_gen.generate_chroma_graphs(normalized_chroma_cqt, "cqt", server_data, song_id)
        r.set(song_id, "70%")
        artifacts_gen.generate_video(server_data, song_id, "cqt", sample_location)
        r.set(song_id, "72%")

    if generate_features:
        artifacts_gen.save_feature_to_server_data("chroma_cqt", server_data, song_id, normalized_chroma_cqt)
        r.set(song_id, "74%")

    del normalized_chroma_cqt
    # op 14    
    normalized_tonnetz = artifacts_gen.transform_to_tonnetz(y, metadata, True)
    r.set(song_id, "76%")
    if generate_video:
        artifacts_gen.generate_tonnetz_graphs(normalized_tonnetz, server_data, song_id)
        r.set(song_id, "78%")
        artifacts_gen.generate_video(server_data, song_id, "tonnetz", sample_location)
        r.set(song_id, "85%")
    if generate_features:
        artifacts_gen.save_feature_to_server_data("tonnetz", server_data, song_id, normalized_tonnetz)
        r.set(song_id, "100%")
    
    del normalized_tonnetz

    return {
        "status": "success",
        "message": "Signal transformed and server data populated successfully.",
        "song_id": song_id,
        "server_data": server_data
    }
