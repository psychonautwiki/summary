extern crate katana;

mod wordnet_stemmer;

use wordnet_stemmer::{WordnetStemmer, NOUN, VERB, ADJ, ADV};

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

const STOP_WORDS: &'static [&'static str] = &[
    "a", "able", "about", "across", "after", "all", "almost", "also",
    "am", "among", "an", "and", "any", "are", "as", "at", "be",
    "because", "been", "but", "by", "can", "cannot", "could", "dear",
    "did", "do", "does", "either", "else", "ever", "every", "for",
    "from", "get", "got", "had", "has", "have", "he", "her", "hers",
    "him", "his", "how", "however", "i", "if", "in", "into", "is",
    "it", "its", "just", "least", "let", "like", "likely", "may",
    "me", "might", "most", "must", "my", "neither", "no", "nor",
    "not", "of", "off", "often", "on", "only", "or", "other", "our",
    "own", "rather", "said", "say", "says", "she", "should", "since",
    "so", "some", "than", "that", "the", "their", "them", "then",
    "there", "these", "they", "this", "tis", "to", "too", "twas",
    "us", "wants", "was", "we", "were", "what", "when", "where",
    "which", "while", "who", "whom", "why", "will", "with", "would",
    "yet", "you", "your"
];

struct Fun {
    stemmer: WordnetStemmer,
    stop_words: HashSet<String>
}

impl Fun {
    fn new () -> Fun {
        let stemmer = WordnetStemmer::new("./dict/").unwrap();

        let stop_words = STOP_WORDS.iter()
                                   .map(|word| word.to_string())
                                   .collect::<HashSet<String>>();

        Fun {
            stemmer,
            stop_words
        }
    }

    fn cut (&self, phrase: &str) -> String {
        let a = self.stemmer.lemma_phrase(NOUN, phrase);
        let a = self.stemmer.lemma_phrase(VERB, &a);
        let a = self.stemmer.lemma_phrase(ADJ, &a);

        self.stemmer.lemma_phrase(ADV, &a)
    }

    fn process_words(&self, phrase: &str) -> String {
        let ref stop_words = self.stop_words;

        phrase
            .to_lowercase()
            .split_whitespace()
            .filter_map(|word| {
                if !stop_words.contains(word) {
                    Some(word)
                } else {
                    None
                }
            })
            .collect::<Vec<&str>>()
            .join(" ")
    }

    fn process_phrases (&self, phrases: &str) -> Vec<Vec<String>> {
        katana::cut(&phrases.to_string())
            .iter()
            .map(|phrase| {
                vec!(self.cut(
                    &self.process_words(
                        &phrase.replace(".", "")
                               .replace(",", "")
                               .replace("\"", "")
                               .replace("'", "")
                    )
                ), phrase.to_string())
            })
            .collect()
    }

    fn summarize (&mut self, phrases: &str, max_phrases: u32) -> Vec<String> {
        let phrases = self.process_phrases(phrases);

        let mut keyword_frequency: HashMap<String, u32> = HashMap::new();

        let cut_phrases =
            phrases.iter()
                   .map(|phrase| {
                        phrase[0].split(" ")
                            .collect::<Vec<&str>>()
                    })
                   .collect::<Vec<Vec<&str>>>();

        let in_phrases =
            phrases.iter()
                   .map(|phrase| phrase[1].clone())
                   .collect::<Vec<String>>();

        /* populate keyword frequency map */
        for phrase in cut_phrases.iter() {
            for word in phrase.iter() {
                let new_keyword_count: u32 = (*keyword_frequency.get(*word).unwrap_or(&0u32)) + 1;

                keyword_frequency.insert(word.to_string(), new_keyword_count);
            }
        }

        let mut phrase_weights: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();

        let mut i:u32 = 0;

        /* populate keyword frequency map */
        for phrase in cut_phrases.iter() {
            let mut weight = 0u32;

            for word in phrase.iter() {
                let word_weight = *keyword_frequency.get(*word).unwrap();

                weight = weight + word_weight;
            }

            /* insert a weight relation: weight -> [phrases, ..] */
            let mut weight_map = match phrase_weights.get(&weight) {
                Some(map) => (*map).clone(),
                None => BTreeSet::new()
            };

            weight_map.insert(i);

            phrase_weights.insert(weight, weight_map);

            i = i + 1;
        }

        let mut out_set: BTreeSet<u32> = BTreeSet::new();

        let mut k = 0u32;

        for (_, weight_set) in phrase_weights.iter().rev() {
            for entry in weight_set.iter() {
                k = k + 1;

                if max_phrases < k {
                    break;
                }

                out_set.insert(*entry);
            }

            if max_phrases < k {
                break;
            }
        }

        out_set
            .iter()
            .map(|entry| in_phrases[*entry as usize].clone())
            .collect()
    }
}

fn main(){
    let phrase = "In this work, we present a novel background subtraction system that uses a deep Convolutional Neural Network (CNN) to perform the segmentation. With this approach, feature engineering and parameter tuning become unnecessary since the network parameters can be learned from data by training a single CNN that can handle various video scenes. Additionally, we propose a new approach to estimate background model from video. For the training of the CNN, we employed randomly 5 percent video frames and their ground truth segmentations taken from the Change Detection challenge 2014(CDnet 2014). We also utilized spatial-median filtering as the post-processing of the network outputs. Our method is evaluated with different data-sets, and the network outperforms the existing algorithms with respect to the average ranking over different evaluation metrics. Furthermore, due to the network architecture, our CNN is capable of real time processing.";

    let mut fun = Fun::new();

    println!("{:?}", fun.summarize(phrase, 3u32));
}