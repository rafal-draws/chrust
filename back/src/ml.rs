#[allow(unused)]
pub mod ml {

    // TODO
    //    http:
    //    - classification results
    //    - model used and training data
    //    - h ow classification was doen
    //    - what the feature is
    //    - how the feature was extracted

    use std::{
        collections::{BTreeMap, HashMap, HashSet},
        error::Error,
        fmt::{self, write, Display},
        fs::File,
        ops::{Div, Mul},
        path::{Path, PathBuf},
    };

    use ndarray::{array, Array2, Array3, ArrayBase, OwnedRepr};
    use ndarray_npy::{ReadNpyError, ReadNpyExt, WriteNpyError};
    use tch::{nn::ModuleT, CModule, Kind, Tensor};

    use crate::db;

    fn load_signal(track_id: String) {}

    #[derive(Debug)]
    pub enum Class {
        Rock,
        HipHop,
        Electronic,
        Pop,
        Classical,
    }

    impl Display for Class {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Class::Rock => write!(f, "{}", "Rock"),
                Class::HipHop => write!(f, "{}", "Hip-Hop"),
                Class::Electronic => write!(f, "{}", "Electronic"),
                Class::Pop => write!(f, "{}", "Pop"),
                Class::Classical => write!(f, "{}", "Classical"),
            }
        }
    }



    #[derive(Debug)]
    pub struct CustomError(String);

    #[derive(Hash, Eq, PartialEq, Debug, Clone)]
    pub enum Feature {
        Ft,
        Mfcc,
        ChromaCens,
        ChromaCqt,
        ChromaStft,
        Spectrogram,
        PowerSpectrogram,
        MelSpectrogram,
        Tonnetz,
    }
    impl Display for Feature {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Feature::ChromaCens => write!(f, "{}", "Chroma (CENS)"),
                Feature::Ft => write!(f, "{}", "Fourier Transform"),
                Feature::Mfcc => write!(f, "{}", "Mel Frequency Cepstral Coefficients (MFCC)"),
                Feature::ChromaCqt => write!(f, "{}", "Chroma (CQT)"),
                Feature::ChromaStft => write!(f, "{}", "Chroma (STFT)"),
                Feature::Spectrogram => write!(f, "{}", "Spectrogram"),
                Feature::PowerSpectrogram => write!(f, "{}", "Power Spectrogram"),
                Feature::MelSpectrogram => write!(f, "{}", "Mel Spectrogram"),
                Feature::Tonnetz => write!(f, "{}", "Tonnetz"),
            }
        }
    }

    impl Feature {
        pub fn all() -> [Feature; 9] {
            [
                Feature::Ft,
                Feature::Mfcc,
                Feature::Tonnetz,
                Feature::ChromaCens,
                Feature::ChromaCqt,
                Feature::ChromaStft,
                Feature::Spectrogram,
                Feature::MelSpectrogram,
                Feature::PowerSpectrogram,
            ]
        }
    }

    #[derive(Debug)]
    pub struct SongClassificationResult {
        pub audio_title: String,
        pub feature_classification_result: Vec<FeatureClassificationResult>,
        pub cum_classification: Vec<f32>,
        pub major_class: Class,
    }

    impl SongClassificationResult {
        pub fn new(instantiated_models: &mut HashMap<Feature, CModule>, song_id: String) -> Self {
            let features = Feature::all();
            
            let classifications: Vec<FeatureClassificationResult> = features
            .iter()
            .map(|feature| FeatureClassificationResult::new(instantiated_models, &feature, song_id.clone()))
            .collect();

            let cum_classification: Vec<f32> = SongClassificationResult::get_cum_classification(&classifications);

            let major_class = SongClassificationResult::get_major_class(&cum_classification).expect("Should be valid at this point");

            Self {
                audio_title: song_id,
                feature_classification_result: classifications,
                cum_classification: cum_classification,
                major_class: major_class
            }


        }

        pub fn get_features_formatted_for_path(&self) -> Vec<FeatureDetail> {
            
            self.feature_classification_result.iter().map(|f| {
                FeatureDetail {
                    folder: FeatureDetail::get_folder(&f.feature),
                    name: FeatureDetail::get_name(&f.feature),
                    short_desc: FeatureDetail::provide_details(&f.feature)
                }
        }).collect::<Vec<FeatureDetail>>()

        }

        fn get_cum_classification(classifications: &Vec<FeatureClassificationResult>) -> Vec<f32> {

                let mut base: [f32; 5] = [0.0; 5];

                let mut total_weight: f32 = 0.0;

                let len = classifications.len().clone() as f32;

                for classification in classifications {
                    

                    let cum_class = classification.avg_classification.clone();

                    cum_class.iter().enumerate().for_each(|(idx, f)| base[idx] += f.mul(&classification.feature_weight));

                    total_weight += classification.feature_weight;
                }
    
                let res: Vec<f32> = dbg!(base).iter().map(|val| dbg!(val) / dbg!(total_weight)).collect();
                dbg!(res)
            
        }

        fn get_major_class(cum_classifications: &Vec<f32>) -> Result<Class, CustomError> {
            let mut biggest_idx: usize = 0;
            let mut biggest_val: f32 = 0.0;
            let _ = cum_classifications.iter().enumerate().for_each(|(idx, class_percent)| {
                if &class_percent >= &&biggest_val {
                    dbg!("class >= &biggest val!\nclass:{:?},biggest_val:{:?}", &class_percent, &biggest_val);
                    biggest_idx = idx;
                    biggest_val = *class_percent;
                }
            });

             match biggest_idx {
                0 => Ok(Class::Rock),
                1 => Ok(Class::HipHop),
                2 => Ok(Class::Electronic),
                3 => Ok(Class::Pop),
                4 => Ok(Class::Classical),
                _ => Err(CustomError("Classification unsuccesfull, wrong major class calculation".to_string()))
            }
        }
    }

    
    pub struct FeatureDetail {
        pub folder: String,
        pub name: String,
        pub short_desc: String
    }


    impl FeatureDetail {

        pub fn get_folder(feature: &Feature) -> String {
            match feature {
                Feature::Ft => "ft",
                Feature::Mfcc => "mfcc",
                Feature::ChromaCens => "cens",
                Feature::ChromaCqt => "cqt",
                Feature::ChromaStft => "stft",
                Feature::Spectrogram => "spectr",
                Feature::MelSpectrogram => "mel_spectr",
                Feature::PowerSpectrogram => "power_spectr",
                Feature::Tonnetz => "tonnetz"
                }.to_string()
        }

        pub fn get_name(feature: &Feature) -> String {
            match feature {
                Feature::Ft => "Short time fourier transform",
                Feature::Mfcc => "MFCC (Mel Frequency Ceptsral Coefficients)",
                Feature::ChromaCens => "Chroma (CENS)",
                Feature::ChromaCqt => "Chroma (CQT)",
                Feature::ChromaStft => "CHROMA (STFT)",
                Feature::Spectrogram => "Spectrogram",
                Feature::MelSpectrogram => "Mel Spectrogram",
                Feature::PowerSpectrogram => "Power Spectrogram",
                Feature::Tonnetz => "Tonnetz"
                }.to_string()
        }


        pub fn provide_details(feature: &Feature) -> String {
            match feature {
                Feature::Ft => r#"
                    <h2>How STFT Works (Conceptually)</h2>
                    <ol>
                    <li>The input audio signal is <strong>divided into overlapping frames</strong> using <code>win_length</code> and <code>hop_length</code>.</li>
                    <li>Each frame is <strong>windowed</strong> using a window function (e.g., Hann) to reduce edge artifacts.</li>
                    <li>A <strong>Fast Fourier Transform (FFT)</strong> is applied to each windowed frame to transform it from the time domain to the frequency domain.</li>
                    <li>The result is a <strong>2D array of complex numbers</strong>:
                        <ul>
                        <li>Each column = spectrum of a frame (frequency content at a point in time)</li>
                        <li>Each row = specific frequency bin (amplitude of that frequency over time)</li>
                        </ul>
                    </li>
                    </ol>
                    "#,
    
                    Feature::ChromaCens => r#"
                    <h2>Chroma CENS (Chroma Energy Normalized Statistics)</h2>
                    <ol>
                    <li>Reduces pitch information into <strong>12 chroma bins</strong> (one per pitch class: C, C#, D, etc.), ignoring octave.</li>
                    <li>Applies <strong>energy normalization</strong> and smoothing over time, making it robust to changes in dynamics and articulation.</li>
                    <li>Useful for identifying <strong>harmonic patterns</strong> and musical similarity.</li>
                    </ol>
                    "#,
    
                    Feature::Mfcc => r#"
                    <h2>MFCC (Mel-Frequency Cepstral Coefficients)</h2>
                    <ol>
                    <li>Transforms audio into a compact representation of its <strong>timbre</strong>.</li>
                    <li>Steps:
                        <ul>
                        <li>Compute Mel Spectrogram (frequency in Mel scale)</li>
                        <li>Apply log transform (log-mel spectrogram)</li>
                        <li>Apply Discrete Cosine Transform (DCT) to get decorrelated coefficients</li>
                        </ul>
                    </li>
                    <li>Commonly used in <strong>speech and music classification</strong>.</li>
                    </ol>
                    "#,
    
                    Feature::ChromaCqt => r#"
                    <h2>Chroma CQT (Constant-Q Transform)</h2>
                    <ol>
                    <li>Like chroma STFT, but uses a <strong>Constant-Q Transform</strong> instead of FFT.</li>
                    <li>Each frequency bin is logarithmically spaced — matching musical pitch perception.</li>
                    <li>Better resolution for <strong>low frequencies</strong>, and pitch-focused tasks.</li>
                    </ol>
                    "#,
    
                    Feature::ChromaStft => r#"
                    <h2>Chroma STFT</h2>
                    <ol>
                    <li>Computes <strong>chroma features</strong> (pitch classes) from a standard STFT spectrogram.</li>
                    <li>Reduces the full frequency spectrum to 12 pitch classes.</li>
                    <li>Good for analyzing <strong>harmonic content</strong>, chords, or key.</li>
                    </ol>
                    "#,
    
                    Feature::Spectrogram => r#"
                    <h2>Spectrogram (Magnitude)</h2>
                    <ol>
                    <li>Represents the audio signal's <strong>frequency content over time</strong>.</li>
                    <li>Computed using the <code>STFT</code>, followed by taking the <code>magnitude</code> (i.e., <code>np.abs</code>).</li>
                    <li>Shows how strong each frequency is at each time step.</li>
                    <li>Useful for <strong>visual inspection</strong> and signal processing tasks.</li>
                    </ol>
                    "#,
    
                    Feature::PowerSpectrogram => r#"
                    <h2>Power Spectrogram</h2>
                    <ol>
                    <li>Like a regular spectrogram, but instead of magnitude, it uses <strong>power</strong>: <code>np.abs(S)**2</code>.</li>
                    <li>Gives more weight to strong frequencies — useful for some machine learning tasks.</li>
                    <li>Can be converted to decibels (log scale) using <code>librosa.power_to_db()</code>.</li>
                    </ol>
                    "#,
    
                    Feature::MelSpectrogram => r#"
                    <h2>Mel Spectrogram</h2>
                    <ol>
                    <li>Transforms a power spectrogram into the <strong>Mel scale</strong>, which aligns better with human pitch perception.</li>
                    <li>More resolution for lower frequencies, less for higher ones — similar to how humans hear.</li>
                    <li>Used in <strong>audio classification</strong>, music tagging, speech recognition, etc.</li>
                    </ol>
                    "#,
    
                    Feature::Tonnetz => r#"
                    <h2>Tonnetz (Tonal Centroid Features)</h2>
                    <ol>
                    <li>Maps chroma vectors to a 6D space representing <strong>tonal relationships</strong>.</li>
                    <li>Captures <strong>harmonic structure</strong> (e.g., consonance, mode) using music theory.</li>
                    <li>Useful for <strong>key detection</strong> and <strong>musical similarity analysis</strong>.</li>
                    </ol>
                    "#,
                }.to_string()
            }


    }

    impl fmt::Display for FeatureDetail {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str(&self.short_desc)
        }
    }

    #[derive(Debug)]
    pub struct FeatureClassificationResult {
        pub feature: Feature,
        pub feature_weight: f32,
        pub avg_classification: Vec<f32>,
        pub weighted_avg_classification: Vec<f32>,
        pub avg_classification_string: Vec<String>,
        pub weighted_avg_classification_string: Vec<String>,
        pub per_frame_classifications: BTreeMap<i64, Vec<f32>>,
    }


    impl FeatureClassificationResult {

        pub fn new(
            instantiated_models: &mut HashMap<Feature, CModule>,
            feature_type: &Feature,
            song_id: String,
        ) -> FeatureClassificationResult {


            let weight: f32 = match &feature_type {
                Feature::Ft => 0.76, // 75% 78% 86% 71% 93%
                Feature::Spectrogram => 0.75,
                Feature::MelSpectrogram => 0.80,
                Feature::PowerSpectrogram => 0.77,
                Feature::Mfcc => 0.69,
                Feature::ChromaStft => 0.45,
                Feature::ChromaCqt => 0.49,
                Feature::ChromaCens => 0.47,
                Feature::Tonnetz => 0.42,
            };

            let feature_tensor =
                load_and_transform_signal(&feature_type, song_id).expect("Should get tensor");


            let model = instantiated_models
                .get_mut(&feature_type)
                .expect("Should be valid Model");



            model.set_eval();




            let classification = model
                .forward_t(&feature_tensor, false)
                .softmax(-1, Kind::Float);

            let mut per_frame_classification: BTreeMap<i64, Vec<f32>> = BTreeMap::new();

            for i in 0..classification.size()[0] {
                let row = Vec::<f32>::try_from(classification.get(i)).expect("Wrong tensor?");
                per_frame_classification.insert(i, row);
            }

            let mut avg_classification: [f32; 5] = [0.0; 5];


            for frame in per_frame_classification.keys() {
                let vec = per_frame_classification
                .get(frame)
                .expect("Should exist");
                
                vec.iter().enumerate().for_each(|(idx, f)| avg_classification[idx] += f);
            }
            

            let avg_classification: Vec<f32> = avg_classification
                .iter()
                .map(|f| f.div(per_frame_classification.len() as f32))
                .collect();

            let weighted_avg_classification: Vec<f32> = avg_classification.clone().iter().map(|x| x.mul(weight)).collect();
            

            let avg_classification_string: Vec<String> = avg_classification
                .iter().map(|x| format!("{:.2}%", x * 100.0))
                .collect();

            let weighted_avg_classification_string: Vec<String> = weighted_avg_classification.clone().iter().map(|x| format!("{:.2}%", x * 100.0)).collect();
            

            Self {
                feature: feature_type.clone(),
                avg_classification: avg_classification,
                feature_weight: weight,
                per_frame_classifications: per_frame_classification,
                weighted_avg_classification: weighted_avg_classification,
                weighted_avg_classification_string,
                avg_classification_string
            }
        }
    }




    /// helper functions

    pub fn get_cmodule_path(feature_type: &Feature) -> &'static Path {
        match feature_type {
            Feature::ChromaCens => std::path::Path::new("/util/chroma_cens.pt"),
            Feature::ChromaCqt => std::path::Path::new("/util/chroma_cqt.pt"),
            Feature::ChromaStft => std::path::Path::new("/util/chroma_stft.pt"),
            Feature::Ft => std::path::Path::new("/util/ft_model.pt"),
            Feature::Spectrogram => std::path::Path::new("/util/mel_spectrogram_model.pt"),
            Feature::MelSpectrogram => std::path::Path::new("/util/mel_spectrogram_model.pt"),
            Feature::PowerSpectrogram => std::path::Path::new("/util/power_spectrogram_model.pt"),
            Feature::Mfcc => std::path::Path::new("/util/mfcc.pt"),
            Feature::Tonnetz => std::path::Path::new("/util/tonnetz.pt"),
        }
    }

    pub fn find_signal_path(feature_type: &Feature, song_id: String) -> PathBuf {
        let features_path = std::path::Path::new(
            &std::env::var("SERVER_DATA").expect("SERVER_DATA should be defined"),
        )
        .join("features")
        .join(song_id);

        let final_path = match feature_type {
            Feature::Tonnetz => features_path.join("tonnetz/tonnetz.npy"),
            Feature::ChromaCens => features_path.join("chroma_cens/chroma_cens.npy"),
            Feature::ChromaCqt => features_path.join("chroma_cqt/chroma_cqt.npy"),
            Feature::ChromaStft => features_path.join("chroma_stft/chroma_stft.npy"),
            Feature::Ft => features_path.join("ft/ft.npy"),
            Feature::MelSpectrogram => features_path.join("mel_spectr/mel_spectr.npy"),
            Feature::PowerSpectrogram => features_path.join("power_spectr/power_spectr.npy"),
            Feature::Spectrogram => features_path.join("spectr/spectr.npy"),
            Feature::Mfcc => features_path.join("mfcc/mfcc.npy"),
        };
        final_path
    }

    pub fn load_and_transform_signal(
        feature_type: &Feature,
        song_id: String,
    ) -> Result<Tensor, Box<dyn Error>> {
        let path = find_signal_path(feature_type, song_id);

        let reader = File::open(path)?;
        let data: ArrayBase<OwnedRepr<f32>, ndarray::Dim<[usize; 3]>> =
            Array3::<f32>::read_npy(reader).expect("Should be able to read to array");

        let shape = data.dim();
        let flattened = data.into_raw_vec_and_offset();
        let tensor = Tensor::try_from(&flattened.0)?
            .reshape(&[shape.0 as i64, shape.1 as i64 * shape.2 as i64]);

        Ok(tensor)
    }

    pub fn instantiate_models(features: [Feature; 9]) -> HashMap<Feature, CModule> {
        let mut model_hm: HashMap<Feature, CModule> = HashMap::new();
        for feature in features {
            model_hm.insert(
                feature.clone(),
                CModule::load(get_cmodule_path(&feature))
                    .expect("Should be able to load the model"),
            );
        }
        model_hm
    }



    mod tests {
        use std::collections::BTreeMap;

        use tch::kind;

        use super::*;
        

        #[test] // TODO
        fn classification_works_for_song() {
            
            unsafe {
                std::env::set_var("SERVER_DATA", "/home/rwd/dev/test_data");
            }

            let mut models = instantiate_models(Feature::all());

            let song_classification = SongClassificationResult::new(&mut models,
                 "8d298e5b-e11a-4ab4-ab38-7149c710a90a-faintofficialmusicvideo[4kupgrade]–linkinpark.mp3".to_string());

            dbg!(song_classification.cum_classification);

            dbg!(song_classification.major_class);

            // let classifications: Vec<Vec<f32>> = song_classification.feature_classification_result.iter().map(|f| f.avg_classification.clone()).collect();

            dbg!(song_classification.feature_classification_result.into_iter().for_each(|x| println!("\n{:?}, avg classification: {:?}, weigth: {:?}, weighted classification: {:?}", x.feature, x.avg_classification, x.feature_weight, x.weighted_avg_classification)));
            

            assert!(true);


        }


        #[test]
        fn classification_ft() {
            unsafe {
                std::env::set_var("SERVER_DATA", "/home/rwd/dev/test_data");
            }
            let mut models = instantiate_models(Feature::all());
            let feature_classification = 
            FeatureClassificationResult::new(&mut models, &Feature::Ft, "8d298e5b-e11a-4ab4-ab38-7149c710a90a-faintofficialmusicvideo[4kupgrade]–linkinpark.mp3".to_string());

            dbg!(&feature_classification.avg_classification);
            dbg!(&feature_classification.weighted_avg_classification);
            
            dbg!(&feature_classification.feature);

        } 


        #[test]
        fn classification_mfcc() {
            unsafe {
                std::env::set_var("SERVER_DATA", "/home/rwd/dev/test_data");
            }
            let mut models = instantiate_models(Feature::all());
            let feature_classification = 
            FeatureClassificationResult::new(&mut models, &Feature::Mfcc, "8d298e5b-e11a-4ab4-ab38-7149c710a90a-faintofficialmusicvideo[4kupgrade]–linkinpark.mp3".to_string());
            assert_eq!(feature_classification.avg_classification.len(), 5 as usize);


        } 


        #[test]
        fn classification_tonnetz() {
            unsafe {
                std::env::set_var("SERVER_DATA", "/home/rwd/dev/test_data");
            }
            let mut models = instantiate_models(Feature::all());
            let feature_classification = 
            FeatureClassificationResult::new(&mut models, &Feature::Tonnetz, "8d298e5b-e11a-4ab4-ab38-7149c710a90a-faintofficialmusicvideo[4kupgrade]–linkinpark.mp3".to_string());

            assert_eq!(feature_classification.avg_classification.len(), 5 as usize);

        } 

        #[test] 
        fn classification_chroma_cens_spectrogram() {
            unsafe {
                std::env::set_var("SERVER_DATA", "/home/rwd/dev/test_data");
            }
            let mut models = instantiate_models(Feature::all());
            let feature_classification = 
            FeatureClassificationResult::new(&mut models, &Feature::ChromaCens, "8d298e5b-e11a-4ab4-ab38-7149c710a90a-faintofficialmusicvideo[4kupgrade]–linkinpark.mp3".to_string());

            assert_eq!(feature_classification.avg_classification.len(), 5 as usize);

        }

        #[test] 
        fn classification_chroma_stft_spectrogram() {
            unsafe {
                std::env::set_var("SERVER_DATA", "/home/rwd/dev/test_data");
            }
            let mut models = instantiate_models(Feature::all());
            let feature_classification = 
            FeatureClassificationResult::new(&mut models, &Feature::ChromaStft, "8d298e5b-e11a-4ab4-ab38-7149c710a90a-faintofficialmusicvideo[4kupgrade]–linkinpark.mp3".to_string());

            assert_eq!(feature_classification.avg_classification.len(), 5 as usize);

        }

        #[test] 
        fn classification_chroma_cqt_spectrogram() {
            unsafe {
                std::env::set_var("SERVER_DATA", "/home/rwd/dev/test_data");
            }
            let mut models = instantiate_models(Feature::all());
            let feature_classification = 
            FeatureClassificationResult::new(&mut models, &Feature::ChromaCqt, "8d298e5b-e11a-4ab4-ab38-7149c710a90a-faintofficialmusicvideo[4kupgrade]–linkinpark.mp3".to_string());

            assert_eq!(feature_classification.avg_classification.len(), 5 as usize);

        } 



        #[test] // works
        fn classification_spectrogram() {
            unsafe {
                std::env::set_var("SERVER_DATA", "/home/rwd/dev/test_data");
            }
            let mut models = instantiate_models(Feature::all());
            let feature_classification = 
            FeatureClassificationResult::new(&mut models, &Feature::Spectrogram, "8d298e5b-e11a-4ab4-ab38-7149c710a90a-faintofficialmusicvideo[4kupgrade]–linkinpark.mp3".to_string());

            assert_eq!(feature_classification.avg_classification.len(), 5 as usize);

        } 

        #[test] // works
        fn classification_mel_spectrogram() {
            unsafe {
                std::env::set_var("SERVER_DATA", "/home/rwd/dev/test_data");
            }
            let mut models = instantiate_models(Feature::all());
            let feature_classification = 
            FeatureClassificationResult::new(&mut models, &Feature::MelSpectrogram, "8d298e5b-e11a-4ab4-ab38-7149c710a90a-faintofficialmusicvideo[4kupgrade]–linkinpark.mp3".to_string());

            assert_eq!(feature_classification.avg_classification.len(), 5 as usize);

        } 

        #[test] // works
        fn classification_power_spectrogram() {
            unsafe {
                std::env::set_var("SERVER_DATA", "/home/rwd/dev/test_data");
            }
            let mut models = instantiate_models(Feature::all());
            let feature_classification = 
            FeatureClassificationResult::new(&mut models, &Feature::PowerSpectrogram, "8d298e5b-e11a-4ab4-ab38-7149c710a90a-faintofficialmusicvideo[4kupgrade]–linkinpark.mp3".to_string());

            assert_eq!(feature_classification.avg_classification.len(), 5 as usize);

        } 


        #[test]
        fn should_load_model() {
            let model = CModule::load(get_cmodule_path(&Feature::Ft))
                .expect("Should be able to load the model");


            assert!(true)
        }

        #[test]
        fn gets_model_path() {
            let path = get_cmodule_path(&Feature::ChromaCens);

            assert_eq!(std::path::Path::new("util/chroma_cens.pt"), path)
        }
    }
}
