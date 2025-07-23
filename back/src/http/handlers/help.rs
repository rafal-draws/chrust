use askama::Template;
use axum::{extract::Path};
use reqwest::StatusCode;

use crate::http::handlers::HtmlTemplate;

pub struct Image {
    pub url: String,
    pub caption: String,
}

#[derive(Template)]
#[template(path = "help.html")]
pub struct HelpPage {
    title: String,
    what: String,
    how: String,
    why: String,
    image: Vec<Image>,
    links: Vec<String>,
}

fn get_data_for_help_page(keyword: String) -> HelpPage {
    match keyword.as_str() {
        "window_function" => HelpPage {
            title: r#"What is a Window Function?"#.to_string(),
    
            what: r#"
    <p>
    A <strong>window function</strong> in signal processing is a mathematical function applied to a segment of a signal before a spectral transformation (like FFT). It tapers the signal's edges toward zero, reducing discontinuities at the segment's boundaries.
    </p>
    
    <p>
    This process minimizes <strong>spectral leakage</strong>, which occurs when a signal is abruptly truncated during analysis. Instead of analyzing raw chunks of the signal, we multiply the segment by a smooth curve (the window) that reduces the start and end to zero or near-zero.
    </p>
    "#.to_string(),
    
            why: r#"
    <p>
    When analyzing finite segments of a signal, sharp edges caused by cutting the signal introduce high-frequency components — this is known as <strong>spectral leakage</strong>.
    </p>
    
    <p>Window functions help by:</p>
    <ul>
        <li><strong>Reducing spectral leakage</strong></li>
        <li>Emphasizing the middle part of a signal</li>
        <li>Improving frequency resolution in transforms</li>
        <li>Enhancing clarity in audio, radar, speech, and vibration analysis</li>
    </ul>
    "#.to_string(),
    
            how: r#"
    <p>
    A window function is simply an array of weights of the same length as the signal segment. Each sample is multiplied by the corresponding window coefficient before performing operations like FFT.
    </p>
    
    <p><strong>Formula:</strong></p>
    
    <p>
    $$
    x_{\text{windowed}}[n] = x[n] \cdot w[n]
    $$
    </p>
    
    <p>Where:</p>
    <ul>
        <li><code>x[n]</code> is the original signal</li>
        <li><code>w[n]</code> is the window function</li>
        <li><code>x_windowed[n]</code> is the result of applying the window</li>
    </ul>
    
    <p>In the case of STFT (Short-Time Fourier Transform), the equation becomes:</p>
    
    <p>
    $$
    X(m, k) = \sum_{n=0}^{N-1} x[n + mH] \cdot w[n] \cdot e^{-j 2\pi k n / N}
    $$
    </p>
    "#.to_string(),
    
            image: vec![
                Image {
                    url: "/static/images/hann_vs_rect.png".to_string(),
                    caption: "Comparison of Rectangular vs. Hann window shapes.".to_string()
                },
                Image {
                    url: "/static/images/window_leakage_demo.png".to_string(),
                    caption: "Frequency response with and without windowing — shows spectral leakage.".to_string()
                }
            ],
    
            links: vec![
                "https://en.wikipedia.org/wiki/Window_function".to_string(),
                "https://docs.scipy.org/doc/scipy/reference/signal.windows.html".to_string(),
                "https://www.mathworks.com/help/signal/ug/windows.html".to_string(),
                "https://ccrma.stanford.edu/~jos/sasp/Windowing.html".to_string()
            ]
        },

"spectrum" => HelpPage {
    title: r#"What is a Spectrum?"#.to_string(),

    what: r#"
A **spectrum** in signal processing is a representation of how the energy or power of a signal is distributed across different frequencies. Instead of viewing a signal in the time domain (how it changes over time), the spectrum shows the signal's content in the frequency domain.

The spectrum is typically obtained by applying a mathematical transformation such as the **Fourier Transform** to a time-domain signal. This process breaks down a complex signal into its constituent sine and cosine waves, revealing which frequencies are present and their relative strengths.
"#.to_string(),

    why: r#"
Analyzing the spectrum of a signal is fundamental in many fields because it allows us to:
- **Identify dominant frequencies** (e.g., pitch in music, or fault frequencies in machinery)
- **Filter unwanted components** (e.g., noise reduction)
- **Visualize and diagnose problems** in sound, vibration, images, and communications
- **Compress and encode data** by focusing on important frequencies

Spectral analysis is used in audio engineering, telecommunications, biomedical engineering, seismology, radar, and more.
"#.to_string(),

    how: r#"
To obtain a spectrum, follow these general steps:

1. **Segment the signal (optional):** For long signals, divide into smaller frames for analysis.
2. **Windowing (optional but recommended):** Apply a window function to reduce spectral leakage.
3. **Transform:** Use the **Fast Fourier Transform (FFT)** or similar algorithm to convert the windowed time-domain signal into the frequency domain.

Formula:
    X[k] = SUM_{n=0}^{N-1} x[n] * exp(-j*2π*k*n/N)

Where:
- `x[n]` is the time-domain signal (possibly after windowing)
- `N` is the length of the segment
- `X[k]` is the complex spectrum at frequency bin `k`
- The **magnitude spectrum** is `|X[k]|`, showing the strength of each frequency component

Typically, the spectrum is plotted as magnitude (or power) versus frequency, providing a clear visual of which frequencies are present and their intensities.
"#.to_string(),

image: vec![
    Image {
        url: "/static/images/spectrum_example.png".to_string(),
        caption: "Example of a signal’s spectrum: amplitude vs. frequency.".to_string(),
    },
    Image {
        url: "/static/images/spectrogram.png".to_string(),
        caption: "Spectrogram: visualizes how the spectrum changes over time.".to_string(),
    },
],

    links: vec![
        "https://www.dspguide.com/ch9/1.htm".to_string(),
        "https://dsp.stackexchange.com/questions/56265/what-do-we-mean-by-spectrum".to_string(),
        "https://www.electronicsforu.com/electronics-projects/electronics-design-guides/understanding-signal-frequency-spectrum".to_string(),
        "https://engineering.purdue.edu/~aae520/tek-scope-spectra-primer.pdf".to_string()
    ]
},

"signal" => HelpPage {
    title: r#"What is a Signal?"#.to_string(),

    what: r#"
A **signal** in engineering and science is a function that conveys information about the behavior or attributes of some phenomenon. Most commonly, signals are functions of time (such as audio waveforms or sensor readings), but they can also depend on space (such as images) or other variables.

Mathematically, a signal is often represented as \( x(t) \) for continuous-time signals or \( x[n] \) for discrete-time signals, where \( t \) is time and \( n \) is an integer index. Signals are the fundamental objects analyzed and processed in fields like signal processing, communications, audio and image analysis, and control systems.
"#.to_string(),

    why: r#"
Signals are central because they are the means by which we represent, store, transmit, and analyze information from the physical world. Understanding signals allows us to:
- **Communicate** (e.g., voice, radio, television, data)
- **Measure and control systems** (e.g., sensors in robotics, feedback in electronics)
- **Analyze behavior** (e.g., medical signals like ECG, seismic waves, financial data)
- **Transform and extract features** for further processing (e.g., in AI, speech recognition, image analysis)

Virtually all modern technology involving data, sensing, or communication relies on understanding and manipulating signals.
"#.to_string(),

    how: r#"
Signals can take many forms:
- **Analog signals:** Continuous in both time and amplitude (e.g., audio waveform, voltage signals)
- **Digital signals:** Discrete in both time and amplitude (e.g., binary data, sampled audio)
- **One-dimensional:** Vary with one independent variable (time, position)
- **Multi-dimensional:** Images (2D), video (3D: 2D + time), other sensor arrays

Typical operations on signals include:
- **Acquisition:** Measuring or capturing a signal (e.g., with a microphone or camera)
- **Sampling:** Converting a continuous signal into a sequence of values (discrete-time)
- **Processing:** Filtering, amplifying, or transforming to extract or enhance information
- **Transmission:** Sending the signal over a medium (wire, air, fiber, etc.)

Example:
    Audio signal (speech):
        - Continuous: x(t) (microphone output)
        - Sampled: x[n] (digital audio in computer)

Visualization:
    - Signals are often plotted as amplitude versus time (1D) or as intensity images (2D).
"#.to_string(),

image: vec![
    Image {
        url: "/static/images/signal_time_domain.png".to_string(),
        caption: "Example of a simple signal plotted in the time domain.".to_string(),
    },
    Image {
        url: "/static/images/analog_vs_digital_signal.png".to_string(),
        caption: "Comparison of analog (continuous) and digital (discrete) signals.".to_string(),
  
    },
],

    links: vec![
        "https://www.dspguide.com/ch1/1.htm".to_string(),
        "https://en.wikipedia.org/wiki/Signal_processing".to_string(),
        "https://www.tutorialspoint.com/digital_signal_processing/digital_signal_processing_signals.htm".to_string(),
        "https://www.electronics-tutorials.ws/io/io_1.html".to_string(),
    ]
},
"frame" => HelpPage {
    title: r#"What is a Frame?"#.to_string(),

    what: r#"
A **frame** in audio signal processing is a short, contiguous segment or block of a longer audio signal, typically consisting of a fixed number of consecutive samples. Frames are used to divide a signal into manageable pieces for analysis or processing—such as transforming, feature extraction, or synthesis.

Frames allow us to analyze how the properties of an audio signal (like pitch, energy, or frequency content) change over time, making them fundamental for tasks like speech recognition, music analysis, and audio effects.
"#.to_string(),

    why: r#"
Audio signals are non-stationary, meaning their characteristics can change rapidly over time. By splitting a signal into overlapping or non-overlapping frames, we can:
- **Track time-varying properties** of the signal
- **Perform localized analysis** (e.g., short-time Fourier transform, MFCC)
- **Detect events** (e.g., speech phonemes, musical notes, onsets)
- **Reduce computational complexity** by working on small blocks
- **Enable real-time processing** and streaming applications

Frame-based analysis is crucial for accurately capturing the dynamic nature of audio signals.
"#.to_string(),

    how: r#"
To extract frames from an audio signal:

1. **Choose a frame length:** Commonly 10–40 ms (e.g., 1024 samples at 44.1 kHz = ~23 ms).
2. **Choose a hop size (stride):** Determines how much the frame window moves each step (often 50% overlap).
3. **Slide the frame window** along the signal, extracting one block at a time.
4. Optionally, **apply a window function** to each frame before further processing.

Formally, for frame index `m`:
    frame_m[n] = x[m * H + n],  where 0 ≤ n < N

Where:
- `x` is the original signal array
- `N` is the frame length (number of samples per frame)
- `H` is the hop size (step between frames)
- `m` is the frame index
- `frame_m[n]` is the nth sample in the mth frame

Frames are often visualized as vertical slices in a spectrogram, showing how frequency content evolves over time.
"#.to_string(),

image: vec![
    Image {
        url: "/static/images/audio_framing.png".to_string(),
        caption: "Illustration of dividing an audio signal into overlapping frames.".to_string(),
    },
    Image {
        url: "/static/images/spectrogram_frames.png".to_string(),
        caption: "Spectrogram with overlaid frame boundaries—each vertical slice is a frame.".to_string(),
    },
],

    links: vec![
        "https://www.dspguide.com/ch9/3.htm".to_string(),
        "https://en.wikipedia.org/wiki/Short-time_Fourier_transform".to_string(),
        "https://haythamfayek.com/2016/04/21/speech-processing-for-machine-learning.html#framing".to_string(),
        "https://www.audiolabs-erlangen.de/resources/MIR/FMP/C1/C1S2_Frames.html".to_string(),
    ]
},

"amplitude" => HelpPage {
    title: r#"What is Amplitude?"#.to_string(),

    what: r#"
**Amplitude** in signal processing refers to the magnitude or strength of a signal at a particular point in time or space. It represents how "large" or "intense" a signal is. For most signals, amplitude is measured as the distance from a reference value (often zero) to the signal's value.

For example, in an audio waveform, amplitude corresponds to the loudness of the sound; in a voltage signal, it represents the voltage level. Amplitude can be positive or negative, but often what matters is its absolute value (magnitude).
"#.to_string(),

    why: r#"
Understanding amplitude is essential because it determines:
- **Signal strength:** Higher amplitude means a stronger (louder/brighter/more intense) signal.
- **Energy and power:** In many cases, the energy carried by a signal is related to the square of its amplitude.
- **Dynamic range:** The difference between the smallest and largest amplitudes, crucial for audio, imaging, and communication quality.
- **Detection and analysis:** Many features and patterns in signals are detected by analyzing changes or peaks in amplitude.

Amplitude is fundamental in audio engineering, communications, instrumentation, and any field dealing with signals.
"#.to_string(),

    how: r#"
Amplitude can be measured in several ways, depending on the signal type:

- **Instantaneous amplitude:** The value of the signal at a specific time, \( x(t) \) or \( x[n] \).
- **Peak amplitude:** The maximum value reached by the signal.
- **Peak-to-peak amplitude:** The difference between the maximum and minimum values.
- **Root mean square (RMS) amplitude:** Represents the effective magnitude of a varying signal, especially for power calculations.

For a continuous signal \( x(t) \), the instantaneous amplitude at time \( t \) is simply \( x(t) \).

Visualization:
- Amplitude is typically shown on the vertical (y) axis in signal plots, with time or another variable on the horizontal (x) axis.
"#.to_string(),

image: vec![
    Image {
        url: "/static/images/amplitude_waveform.png".to_string(),
        caption: "A waveform showing amplitude as the height of the signal from the center line.".to_string(),
    },
    Image {
        url: "/static/images/peak_vs_rms.png".to_string(),
        caption: "Comparison of peak, peak-to-peak, and RMS amplitude for a signal.".to_string(),
    },
],

    links: vec![
        "https://www.dspguide.com/ch2/2.htm".to_string(),
        "https://en.wikipedia.org/wiki/Amplitude".to_string(),
        "https://www.electronics-tutorials.ws/accircuits/acp_2.html".to_string(),
        "https://www.splmeter.com/post/what-is-amplitude-in-audio".to_string(),
    ]
},

"artifact" => HelpPage {
    title: r#"What is an Artifact?"#.to_string(),

    what: r#"
An **artifact** in signal processing is an unwanted or spurious component that appears in a signal as a result of the data acquisition, processing, or transmission process. Artifacts are not part of the original, true signal and can distort analysis or interpretation.

Artifacts can be introduced by the signal capture device (like sensors or microphones), the environment, or digital processing steps such as filtering, compression, or transformation. They may appear as sudden spikes, glitches, echoes, distortions, or repetitive patterns that do not correspond to the actual phenomenon being measured.
"#.to_string(),

    why: r#"
Understanding and managing artifacts is essential because they can:
- **Interfere with analysis** by masking or mimicking real features of the signal
- **Reduce measurement accuracy** in scientific, medical, or engineering applications
- **Lead to misinterpretation** (e.g., false positives in diagnostics, degraded audio/image quality)
- **Affect further processing** like feature extraction, classification, or enhancement

Careful detection and removal (or mitigation) of artifacts are crucial for reliable results in signal processing.
"#.to_string(),

    how: r#"
Artifacts can arise from multiple sources and take various forms:

**Common sources:**
- **Sensor limitations:** Electrical noise, calibration errors, saturation
- **Environmental interference:** Power line hum, electromagnetic interference, movement artifacts
- **Digital processing:** Quantization noise, aliasing, compression artifacts, ringing from filters

**Detection and mitigation strategies:**
- **Visual inspection:** Plotting the signal and looking for anomalies
- **Statistical analysis:** Identifying outliers or non-typical patterns
- **Filtering:** Applying notch, median, or band-stop filters to remove known types of artifacts
- **Signal modeling:** Using expected signal properties to distinguish and correct artifacts

Example:
    In EEG (brainwave) recordings, eye blinks appear as large, slow artifacts. In compressed audio, “pre-echo” or “ringing” can result from aggressive compression or poor filter design.
"#.to_string(),

image: vec![
    Image {
        url: "/static/images/artifact_eeg.png".to_string(),
        caption: "EEG signal with a visible eye-blink artifact.".to_string(),
    },
    Image {
        url: "/static/images/compression_artifact_audio.png".to_string(),
        caption: "Audio waveform showing distortion due to compression artifacts.".to_string(),
    },
],

    links: vec![
        "https://en.wikipedia.org/wiki/Artifact_(signal_processing)".to_string(),
        "https://www.ncbi.nlm.nih.gov/pmc/articles/PMC3445644/".to_string(),
        "https://www.frontiersin.org/articles/10.3389/fnins.2020.594565/full".to_string(),
        "https://www.dspguide.com/ch16/3.htm".to_string(),
    ]
},

        _ => HelpPage { 
            title: r#"404 content not found :("#.into(),
            what: r#"pass"#.to_string(),
            how: r#"pass"#.to_string(),
            why: r#"pass"#.to_string(),
            image: vec![
                Image {
                    url: "location".to_string(), 
                    caption: "description".to_string()
                }],
            links: vec![r#"pass"#.to_string()],
             }
    }
}

pub async fn help(Path(keyword): Path<String>) -> Result<HtmlTemplate<HelpPage>, StatusCode> {
    let template = get_data_for_help_page(keyword.replace(" ", "_"));
    Ok(HtmlTemplate(template))
}
