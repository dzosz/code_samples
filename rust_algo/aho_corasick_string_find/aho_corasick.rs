use std::collections::VecDeque;

fn calc_prefix(s : String) -> Vec<usize> {
    let mut result = vec![0; s.len()];
    
    let bytes = s.as_bytes();
    
    let mut k : usize = 0;
    for i in 1..s.len() {
        let was = k;
        // backtrack K
        while k > 0 && bytes[i] != bytes[k] {
            k = result[k-1];
        }
        //println!("{}) {}->{} {} {:?}", i, was, k, bytes[i] as char, result);
        if bytes[k] == bytes[i] {
            k += 1;
        }
        result[i] = k;
    }
    println!("prefix {:?}", s);
    println!("{:?}", result);
    result
}

fn kmp(t : String, pattern : String) -> i32 {
    let result = calc_prefix(pattern.clone());
    println!("{:?}", t);
    
    let text = t.as_bytes();
    let pattern_bytes = pattern.as_bytes();
    
    let mut max_prefix_len = 0;
    
    for i in 0..text.len() {
        let was = max_prefix_len;
        while max_prefix_len > 0 && text[i] != pattern_bytes[max_prefix_len] {
            println!("while {}", result[max_prefix_len-1]);
            max_prefix_len = result[max_prefix_len-1];
        }
        println!("{}) {}->{} {} {:?}", i, was, max_prefix_len, text[i] as char, result);
        if pattern_bytes[max_prefix_len] == text[i] {
            max_prefix_len += 1;
        }
        if max_prefix_len == pattern.len() {
            // occurs at index
            return (i - pattern.len() + 1) as i32;
            //max_prefix_len = result[max_prefix_len-1];
        }
        
    }
    return -1;
}

const CHAR_MAX : usize = 26;

struct AhoCorasick {
    trie : Vec<Vertex>,
    recalculated : bool,
}

// naive string searching has complexity of N*M+L where N=text length, M=num patterns, L pattern
// length
// Aho Corasick has complexity N+L+Z where Z=count of matches
// based on https://www.toptal.com/algorithms/aho-corasick-algorithm
impl AhoCorasick {
    fn new() -> AhoCorasick {
        let mut obj = AhoCorasick { trie : vec![Vertex::new()], recalculated : false };
        obj.trie.push(Vertex::new());
        obj
    }
    fn add_string(&mut self, new_str : &String) {
        let mut vertex_id : usize = 0;
        for ch in new_str.chars() { 
            let pos  = ch as usize - 'a' as usize;
            
            if self.trie[vertex_id].next[pos].is_none() {
                self.trie[vertex_id].next[pos] = Some(self.trie.len());
                //println!("adding char {}", ch);
                let mut tmp_new = Vertex::new();
                tmp_new.parent = vertex_id;
                //tmp_new.parent_pos = ch as i32;
                tmp_new.parent_pos = pos;
                self.trie.push(tmp_new);
            }
            vertex_id = self.trie[vertex_id].next[pos].unwrap();
        }
        self.trie[vertex_id].is_leaf = true; // this is the end of the string
        self.recalculated = false;
    }

    fn count_matches(&self, text : &String) -> usize {
        //self.recalculate(); // not mut
        assert!(self.recalculated);

        let mut result : usize = 0;

        let mut current_state = 0;

        for ch in text.chars() {
            if char::is_whitespace(ch) {
                continue;
            }
            println!("searching for {}", ch );
            let pos = (ch as u8 - 'a' as u8) as usize;
            loop {
                println!("cur_state for ({})", current_state);
                if let Some(v) = self.trie[current_state].next[pos] {
                    current_state = v;
                    break;
                }
                // otherwise jump by suffix links
                if current_state == 0 { 
                    break;
                }
                current_state = self.trie[current_state].link.unwrap();
            }
            let mut check_state = current_state;
            println!("check_state =>{}",  check_state);

            loop {
                // checking all words that we can get from current prefix;
                check_state = self.trie[check_state].exit_link.unwrap();
                println!("check_state update to exit {}", check_state);

                if check_state == 0 {
                    break;
                }
                println!("MATCH");
                result += 1; // MATCH

                // let index_of_match = j + 1 - wordsLenght[self.trie[check_state].wordID];
                check_state = self.trie[check_state].link.unwrap();
                println!("check_state move to link {}",  check_state);
            }
        }
        result
    }

    fn recalculate(&mut self) {
        if self.recalculated {
            return;
        }
        self.recalculated = true;
        let mut queue = VecDeque::<usize>::new();
        queue.push_front(0);

        // BFS
        while let Some(i) = queue.pop_front() {
            if i == 0 { // root 
                self.trie[i].link = Some(0);
                self.trie[i].exit_link = Some(0);
            } else if self.trie[i].parent == 0 { // 1st level
                self.trie[i].link = Some(0);
                if self.trie[i].is_leaf {
                    self.trie[i].exit_link = Some(i);
                } else {
                    self.trie[i].exit_link = self.trie[self.trie[i].link.unwrap()].exit_link;
                }
            }
            else { // deeper levels
                let mut cur_better_vertex = self.trie[self.trie[i].parent].link.unwrap();
                let ch_vertex = self.trie[i].parent_pos;

                loop {
                    if self.trie[cur_better_vertex].next[ch_vertex].is_some() {
                        self.trie[i].link = self.trie[cur_better_vertex].next[ch_vertex];
                        break;
                    }
                    if cur_better_vertex == 0 {
                        self.trie[i].link = Some(0);
                        break;
                    }
                    cur_better_vertex = self.trie[cur_better_vertex].link.unwrap(); // go back to suf link
                }

                // When we complete the calculation of the suffix link for the current
                // vertex, we should update the link to the end of the maximum length word
                // that can be produced from the current substring.
                self.trie[i].exit_link = if self.trie[i].is_leaf { Some(i) } else { self.trie[self.trie[i].link.unwrap()].exit_link };
            }
            self.trie[i].next.iter().filter(|x| x.is_some()).for_each(|x| queue.push_back(x.unwrap()));
        }
    }

    fn debug(&self) { // dfs
        for (i, elem) in self.trie.iter().enumerate() {
            print!("{}) ", i);
            self.get_next(elem);
            println!("");
        }
    }

    fn get_next(&self, elem : &Vertex) {
        print!("{}-> [{}", (elem.parent_pos + 'a' as usize) as u8 as char, if elem.is_leaf { " LEAF" } else { " ----" } );
        print!(" link={} ", if let Some(x) = elem.link { x.to_string() } else { String::from("-") } );
        print!("exit={}", if let Some(x) = elem.exit_link { x.to_string() } else { String::from("-") } );
        for (i, next) in elem.next.iter().enumerate() {
            if next.is_some() {
                let idx = (i as usize + 'a' as usize) as u8 as char;
                print!(" {}->", idx);
                self.get_next(&self.trie[next.unwrap() as usize]);
            }
        }
        print!(" ]");
    }
}

struct Vertex {
    next : [Option<usize>; CHAR_MAX],
    is_leaf : bool,
    parent : usize,
    parent_pos : usize, 
    link : Option<usize>,
    exit_link : Option<usize>,
    //transitions : [usize; CHAR_MAX]

}

impl Vertex {
    fn new() -> Vertex {
        Vertex { next: [None; CHAR_MAX], is_leaf : false, parent:0, parent_pos: 0, link:None, exit_link : None,
        //transitions:[-1; CHAR_MAX]
        }
    }
}

fn check_kmp() {
    let s = String::from("hu hu hu hu huj huh");
    let pattern = String::from("hu huh");
    let pattern_len = pattern.len();
    let idx = kmp(s, pattern);
    println!("idx {}-{}", idx, idx+pattern_len as i32);
}

fn check_aho()  {
    let mut ah = AhoCorasick::new();
    ah.add_string(&mut String::from("a"));
    ah.add_string(&mut String::from("b"));
    ah.add_string(&mut String::from("c"));
    //ah.add_string(&mut String::from("ba"));

    ah.recalculate();
    ah.debug();

    let text = String::from("bac");
    let matches = ah.count_matches(&text);
    println!("aho-corasick matches: {}", matches);
}
    

fn main() {
    //check_kmp();
    check_aho();
}
