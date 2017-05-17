/* phrase cutter */
mod katana;

/* Stemming & word type detection */
mod wordnet_stemmer;

use self::wordnet_stemmer::{WordnetStemmer, NOUN, VERB, ADJ, ADV};

use std::env;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

const STOP_WORDS: &'static [&'static str] = &["a", "able", "about", "across", "after", "all", "almost", "also", "am", "among", "an", "and", "any", "are", "as", "at", "be", "because", "been", "but", "by", "can", "cannot", "could", "dear", "did", "do", "does", "either", "else", "ever", "every", "for", "from", "get", "got", "had", "has", "have", "he", "her", "hers", "him", "his", "how", "however", "i", "if", "in", "into", "is", "it", "its", "just", "least", "let", "like", "likely", "may", "me", "might", "most", "must", "my", "neither", "no", "nor", "not", "of", "off", "often", "on", "only", "or", "other", "our", "own", "rather", "said", "say", "says", "she", "should", "since", "so", "some", "than", "that", "the", "their", "them", "then", "there", "these", "they", "this", "tis", "to", "too", "twas", "us", "wants", "was", "we", "were", "what", "when", "where", "which", "while", "who", "whom", "why", "will", "with", "would", "yet", "you", "your"];

pub struct Summary {
    stemmer: WordnetStemmer,
    stop_words: HashSet<String>,
}

impl Summary {
    pub fn new() -> Summary {
        let dict_path = &env::var("WORDNET_PATH").unwrap_or("./dict/".to_string());
        let stemmer = WordnetStemmer::new(dict_path).unwrap();

        let stop_words = STOP_WORDS
            .iter()
            .map(|word| word.to_string())
            .collect::<HashSet<String>>();

        Summary {
            stemmer,
            stop_words,
        }
    }

    fn cut(&self, phrase: &str) -> String {
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
            .filter_map(|word| if !stop_words.contains(word) {
                            Some(word)
                        } else {
                            None
                        })
            .collect::<Vec<&str>>()
            .join(" ")
    }

    fn process_phrases(&self, phrases: &str) -> Vec<Vec<String>> {
        katana::cut(&phrases.to_string())
            .iter()
            .map(|phrase| {
                vec!(self.cut(
                    &self.process_words(
                        &phrase.replace(".", "")
                               .replace(",", "")
                               .replace("\"", "")
                               .replace("”", "")
                               .replace("’", "")
                               .replace("‘", "")
                               .replace("“", "")
                               .replace("'", "")
                    )
                ), phrase.to_string())
            })
            .collect()
    }

    pub fn summarize(&mut self, phrases: &str, max_phrases: u32) -> (Vec<String>, Vec<String>) {
        let phrases = self.process_phrases(phrases);

        let mut keyword_frequency: HashMap<String, u32> = HashMap::new();

        let cut_phrases = phrases
            .iter()
            .map(|phrase| phrase[0].split(" ").collect::<Vec<&str>>())
            .collect::<Vec<Vec<&str>>>();

        let in_phrases = phrases
            .iter()
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

        let mut i: u32 = 0;

        /*
            populate keyword frequency map

            Algorithm:
                for each word in phrase
                    build weight from
                        absolute keyword frequency
                        word type multiplier
                    insert offset into weight map at index [weight]

            Trivia:
                For a set [a, b, c] (where a, b, c are words in a phrase)
                compute weight and obtain index, so that a sorted map
                {weight1: [a], weight2: [b, c], ..} can be built.

                This allows for more space-efficient tracking of top
                phrases and for a trivial top-down iteration.
        */

        for phrase in cut_phrases.iter() {
            let mut weight = 0u32;

            for word in phrase.iter() {
                let word_weight = *keyword_frequency.get(*word).unwrap();

                /*
                    ~~Algorithm draft~~

                    get word frequency
                    determine word type
                    apply penalty:
                        noun: x6
                        noun + verb: x3
                        verb: x2
                        verb + {adj, adv}: x1
                        adj, adv: x1
                */

                let multiplier = {
                    let (is_noun, is_verb, is_adj, is_adv) = self.stemmer.word_type(word);

                    if is_noun {
                        if is_verb || is_adj || is_adv {
                            3u32
                        } else {
                            6u32
                        }
                    } else if is_verb {
                        if !(is_adj || is_adv) { 2u32 } else { 1u32 }
                    } else {
                        1u32
                    }
                };

                weight = weight + (word_weight * multiplier);
            }

            /* insert a weight relation: weight -> [phrases, ..] */
            let mut weight_map = match phrase_weights.get(&weight) {
                Some(map) => (*map).clone(),
                None => BTreeSet::new(),
            };

            weight_map.insert(i);

            phrase_weights.insert(weight, weight_map);

            i = i + 1;
        }

        /*
            From BTreeMap<weight: u32, phrases: Vec<u32>> map top max_phrases
            into a BTreeSet (sorted).
        */

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

        /*
            build output tuple: (Vec<String>, Vec<String>) -- (phrases, keywords)
        */
        (out_set
             .iter()
             .map(|entry| in_phrases[*entry as usize].clone())
             .collect(),
         {
             let mut kf_tuples = keyword_frequency
                 .iter()
                 .map(|(k, v)| (k.to_owned(), *v))
                 .collect::<Vec<(String, u32)>>();

             kf_tuples.sort_by(|a, b| a.1.cmp(&b.1));

             kf_tuples
                    .iter()
                    // filter out non-nouns
                    // map to strings
                    .filter_map(move |&(ref a, _)|
                        match self.stemmer.word_type(a) {
                        //   noun  verb   adj.   adv.
                            (true, false, false, false) =>
                                Some(a.to_string()),
                            _ => None
                        }
                    )
                    .collect()
         })
    }
}
