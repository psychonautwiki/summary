extern crate regex;

mod katana;
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

    fn summarize (&mut self, phrases: &str, max_phrases: u32) -> (Vec<String>, Vec<String>) {
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
                        if !(is_adj || is_adv) {
                            2u32
                        } else {
                            1u32
                        }
                    } else {
                        1u32
                    }
                };

                weight = weight + (word_weight * multiplier);
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
        (
            out_set
                .iter()
                .map(|entry| in_phrases[*entry as usize].clone())
                .collect(),
            keyword_frequency
                .keys()
                // copy and take ownership of &str
                .filter_map(|word|
                    Some(word.to_owned().to_string())

                    // TODO: extend
                )
                // build Vec<String> from mapped Vec<&'[temp] str>
                .collect()
        )
    }
}

fn main(){
    let phrase = r#"
MESA, Arizona—Since the dawn of the space age NASA and other agencies have spent billions of dollars to reconnoiter Mars—assailing it with spacecraft flybys, photo-snapping orbiters and landers nose-diving onto its surface. The odds are good, many scientists say, for the Red Planet being an extraterrestrial address for alien life—good enough to sustain decades’ worth of landing very expensive robots to ping it with radar, zap it with lasers, trundle across its terrain and scoop up its dirt. Yet against all odds (and researchers’ hopes for a watershed discovery), Mars remains a poker-faced world that holds its cards tight. No convincing signs of life have emerged. But astrobiologists continue to, quite literally, chip away at finding the truth.
As the search becomes more heated (some would say more desperate), scientists are entertaining an ever-increasing number of possible explanations for Martian biology as a no-show. For example, could there be a “cover up” whereby the harsh Martian environment somehow obliterates all biosignatures—all signs of past or present life? Or perhaps life there is just so alien its biosignatures are simply unrecognizable to us, hidden in plain view.
Of course, the perplexing quest to find life on Mars may have a simple solution: It’s not there, and never was. But as the proceedings of this year’s Astrobiology Science Conference held here in April made clear, life-seeking scientists are not giving up yet. Instead, they are getting more creative, proposing new strategies and technologies to shape the next generation of Mars exploration.
Talk about looking for Martians and you inevitably talk about water, the almost-magical liquid that sustains all life on Earth and seems to have served as an indispensable kick-starter for biology in our planet’s deepest past. “It all started out with ‘follow the water;’ not necessarily ‘follow the life’…but ‘follow one of the basic requirements for living systems,’” says Arizona State University geologist Jack Farmer, referring to NASA’s oft-repeated mantra for Martian exploration. “There are many indications of water on Mars in the past, perhaps reservoirs of water in the near subsurface as well,” he says. “But what is the quality of that water? Is it really salty—too salty for life?”
Without liquid water, Farmer points out, one would naively think organisms cannot function. The reality may be more complex: on Earth, some resilient organisms such as tardigrades can enter a profound, almost indefinite state of hibernation when deprived of moisture, preserving their desiccated tissues but neither growing nor reproducing. It is possible, Farmer says, that Martian microbes could spend most of their time as inert spores “waiting for something good to happen,” only springing to life given the right and very rare conditions. Certain varieties of Earthly “extremophiles”—microbes that live at extremes of temperature, pressure, salinity and so on—exhibit similar behavior.
Farmer says there is as yet no general consensus about the best way to go about life detection on the Red Planet. This is due in no small part to the runaway pace of progress in biotechnology, which has led to innovations such as chemistry labs shrunken down to fit on a computer chip. These technologies “have been revolutionizing the medical field, and have now started to enter into concepts for life detection on Mars,” he explains. Things move so fast that today’s best technology for finding Martian biology may be tomorrow’s laughably obsolete dead-end.
But no matter how sophisticated a lab on a chip might be, it won’t deliver results if it is not sent to the right place. Farmer suspects that seriously seeking traces of life requires deep drilling on Mars. “I basically think we’re going to have to gain access to the subsurface and look for the fossil record,” he explains. But discovering a clear, unambiguous fossil biosignature on Mars would also raise a red flag. “We probably would approach the future of Mars exploration—particularly accessing habitable zones of liquid water in the deep subsurface—more cautiously, because life could still be there. So planetary protection would be taken very seriously,” he says. (“Planetary protection” is the term scientists commonly use for precautions to minimize the chance of biological contamination between worlds. Think of it not so much in terms of bug-eyed aliens running rampant on Earth but of billion-dollar robots finding “Martians” that prove to only be hardy bacterial hitchhikers imported from our own world).
Like-minded about deep diving on Mars is Penelope Boston, director of the NASA Astrobiology Institute at the agency’s Ames Research Center. “That’s my bias,” she says. “Given Mars’ current state, with all the challenging surface manifestations of dryness, radiation and little atmosphere, the best hope for life still extant on Mars is subsurface.” The subsurface, she says, might also offer better chances of preserving past life—that is, of fossils, even if only of single-celled organisms.
The planet’s depths hold the potential for harboring liquid water under certain circumstances, Boston thinks. But how far down might that water be? “I suspect it’s pretty far…and how we get to it, that’s a whole other kettle of fish,” she says. Over the years scientists have estimated the average depth of the planet’s possible liquid reservoirs as anywhere between tens of meters to kilometers. Then again, recent observations from orbiters have revealed mysterious dark streaks that seasonally flow down the sunlit sides of some Martian hillsides and craters. These “recurring slope lineae” could conceivably be brines of liquid water fed by aquifers very close to the surface, some researchers say.
Such lingering uncertainties emerge from the indirect and scattered nature of our studies of Mars, and ensure that any argument for life there is based solely on circumstantial information, Boston notes. “Each individual piece of evidence is, on its own merits, weak,” she says. Only by amassing a diverse suite of independent measurements can a well-built case for life on Mars be made, she says: “In my opinion, we can’t make that strong case unless we push to do all of those measurements on exactly the same precise spot. We don’t do that because it’s very difficult, but it’s something to aspire to.” Despite decades of sending costly hardware to Mars, Boston believes that what is still missing is a sense of harmony between instruments, allowing them to work together to support a search for alien life. “I think that the precise requirements of a really robust claim of life at the microscopic scale require us to push on further,” she notes.
Attendees at the astrobiology meeting in Arizona showcased an assortment of high-tech devices for next-generation exploration, ranging from microfluidic “life analyzers” and integrated nucleic acid extractors for studying “Martian metagenomics” to exquisitely sensitive, miniaturized organic chemistry labs for spotting tantalizing carbon compounds and minerals at microscopic scales. Missing from the mix, however, was any solid consensus on how these and other tools could all work together to provide a slam-dunk detection of life on Mars.
Some scientists contend a new kind of focus is sorely needed. Perhaps the pathway to finding any Martians lurking in the planet’s nooks and crannies is to learn where exactly on Mars those potentially life-nurturing niches exist, and how they change over the course of days, months and years rather than over eons of geologic time. That is, to find homes for extant life on Mars today, researchers should probably not just be studying the planet’s long-term climate but also its day-to-day weather.
“Right now we’re sort of shifting gears. Once you’ve found out that a planet is habitable, then the next question is, ‘Was there life?’—so it’s a completely different ball game,” says Nathalie Cabrol, director of the Carl Sagan Center at the SETI Institute. “On Mars you cannot look for life with the tools that have been looking for habitability of that planet,” she argues. “We should be looking for habitats and not habitable environments. You are dealing on Mars with what I call extremophile extreme environments on steroids,” she says, “and you don’t look for microbial life with telescopes from Mars orbit.”
Cabrol advocates making an unprecedentedly robust, high-resolution study of environmental variability on Mars by peppering its surface with weather stations. Sooner or later telltale signs of the possible whereabouts of extant life may emerge from the resulting torrents of data. “Today’s environment on that planet is a reflection of something in the past,” she says, and planting numbers of automated stations on Mars does not need to be expensive. “This is of interest not only to astrobiology but to human exploration. The first thing you want to know is what the weather is like,” she says, adding, “Right now we’re not equipped to do this and I’m not saying it’s going to be easy to look for extant life. I’m not saying what we’re doing now is wrong. Whatever we put on the ground we are learning. But there is variability on Mars. You go up or down one meter, things change. Habitats at a microscopic level can happen at the scale of a slope. It can happen at the scale of a rock!”
“I think Mars offers us the highest chance of finding life” somewhere beyond Earth, says Dirk Schulze-Makuch, a planetary scientist at Technical University of Berlin in Germany. But, like Boston and others, he maintains confirmation of life will only come from multiple “layers of proof” that have to be consistent with one another. “We really need at least four different kinds of methods,” he says. “My point is that there’s no slam-dunk. We need several instruments. You have to build a case, and right now we can do better…unless the biosignature through a microscope is waving hello.” The trouble, he adds, is that too-stringent planetary protection rules may preclude getting the evidence necessary for that proof. “We have the technology to go to places where there could be life,” he says. “But we can’t go to certain areas on Mars, like recurring slope lineae or…under patches of ice. It seems to be ridiculous.”
Indeed, Schulze-Makuch speculates planetary protection may be a lost cause for Mars—or at least a misguided endeavor. It may even be that any Martian microbes are actually Earth’s long-lost cousins. Or, conversely, Mars rather than Earth is really the sole site of biogenesis in our solar system. Both scenarios are possible, considering that single-celled organisms can likely survive world-shattering impacts and the subsequent interplanetary voyages if embedded in ejected shards of rock that could fall elsewhere as meteorites. Innumerable impacts of this scale battered the solar system billions of years ago, potentially blasting biological material between neighboring worlds. On balance, Schulze-Makuch says, “the chances are higher that we are Martians.”
"#;

    let mut fun = Fun::new();

    println!("{:?}", fun.summarize(phrase, 4u32));
}