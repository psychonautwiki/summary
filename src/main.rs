extern crate katana;

mod wordnet_stemmer;

use wordnet_stemmer::{WordnetStemmer, NOUN, VERB, ADJ, ADV};

use std::collections::HashSet;

const STOP_WORDS: &'static [&'static str] = &["a", "able", "about", "across", "after", "all", "almost", "also", "am", "among", "an", "and", "any", "are", "as", "at", "be", "because", "been", "but", "by", "can", "cannot", "could", "dear", "did", "do", "does", "either", "else", "ever", "every", "for", "from", "get", "got", "had", "has", "have", "he", "her", "hers", "him", "his", "how", "however", "i", "if", "in", "into", "is", "it", "its", "just", "least", "let", "like", "likely", "may", "me", "might", "most", "must", "my", "neither", "no", "nor", "not", "of", "off", "often", "on", "only", "or", "other", "our", "own", "rather", "said", "say", "says", "she", "should", "since", "so", "some", "than", "that", "the", "their", "them", "then", "there", "these", "they", "this", "tis", "to", "too", "twas", "us", "wants", "was", "we", "were", "what", "when", "where", "which", "while", "who", "whom", "why", "will", "with", "would", "yet", "you", "your"];

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

    fn remove_stopwords(&self, phrase: &str) -> String {
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

    fn process (&self, phrases: &str) {
        let lemma: Vec<String> =
            katana::cut(&phrases.to_string())
                .iter()
                .map(|phrase| {
                    self.cut(
                        &self.remove_stopwords(
                            &phrase.replace(".", "")
                                   .replace(",", "")
                                   .replace("\"", "")
                                   .replace("'", "")
                        )
                    )
                })
                .collect();

        println!("{:?}", lemma);

        loop {

        }
    }
}

fn main(){
    let phrase = "Anyways, what is the criterion for inviting someone to this room again? Any thoughts on inviting a user? He seems to be pretty dedicated to the site, has been here for a while, I see him almost every day and he follows proper procedures re: edit summaries and such.";

    let fun = Fun::new();

    fun.process(phrase);
}