use structopt::StructOpt;
use std::collections::HashSet;
use sqb::hash40::Hash40;
use sqb::hash40;

#[derive(StructOpt)]
struct Args {
    files: Vec<String>
}

fn main() {
    let args = Args::from_args();

    sqb::hash40::set_labels(
        sqb::hash40::read_labels("sqb_labels.txt").unwrap()
    );

    let mut hashes = HashSet::<Hash40>::new();

    if args.files.len() > 0 {
        for file in args.files {
            let sqb = sqb::open(&file).unwrap();

            let check = |hash: Hash40|{
                if hash.to_string().starts_with("0x") && hash.strlen() != 0 {
                    eprintln!("0x{:010X} = {:?}", hash.0, file);
                }
            };

            for seq in sqb.sequences {
                hashes.insert(seq.id);
                check(seq.id);
                for sound in seq.sounds {
                    hashes.insert(sound.id);
                    check(sound.id);
                }
            }
        }

        let hashes: Vec<_> = hashes.iter().map(|h| format!("0x{:010X}", h.0)).collect();
        std::fs::write(
            "all_sqb_hashes.txt",
            hashes.join("\n")
        ).unwrap();
    } else {
        let all_hashes = std::fs::read_to_string(
            "all_sqb_hashes.txt"
        ).unwrap();

        hashes = all_hashes.split("\n").map(|h| hash40::Hash40::from_hex_str(h).unwrap()).collect();
    }

    let cracked: Vec<String>
        = hashes
            .iter()
            .filter_map(|hash|{
                let s = hash.to_string();
                if s.starts_with("0x") || hash.strlen() == 0{
                    None
                } else {
                    Some(s)
                }
            })
            .collect();

    let uncracked: HashSet<Hash40>
        = hashes
            .into_iter()
            .filter_map(|hash|{
                if hash.to_string().starts_with("0x") && hash.strlen() != 0 {
                    Some(hash)
                } else {
                    None
                }
            })
            .collect();

    eprintln!("uncracked: {}", uncracked.len());

    let mut just_cracked: HashSet<String> = HashSet::new();

    for s in cracked {
        let parts = s.split("_").collect::<Vec<_>>();
        let character = if parts[0] == "vc" || parts[0] == "seq" {
            Some(parts[1])
        } else {
            None
        };

        for substr in 
            (0..parts.len()-1)
                .map(|i|{
                    (i+1..parts.len())
                        .map(|j| parts[i..j].join("_"))
                        .collect::<Vec<_>>()
                })
                .flatten()
        {
            let mut check = |substr: String|{
                let hash = hash40::to_hash40(&substr);
                if uncracked.contains(&hash) {
                    just_cracked.insert(substr);

                }
            };

            macro_rules! check_fmts {
                ($($fmt:literal),*) => {
                    $(
                        check(format!($fmt, substr));
                    )*
                }
            }

            check_fmts!(
                "seq_{}",
                "seq_{}_rnd",
                "seq_{}_rnd_jump",
                "seq_{}_rnd_jump02",
                "seq_{}_rnd_ottotto",
                "seq_{}_rnd_damage",
                "seq_{}_rnd_damagefly",
                "seq_{}_rnd_damagemid",
                "seq_{}_rnd_attack01",
                "seq_{}_rnd_final",
                "seq_{}_rnd_special_n2",
                "{}_appeal01",
                "{}_appeal02",
                "{}_appeal03",
                "{}_appeal04",
                "{}_appeal05",
                "{}_appeal06",
                "{}_appeal07",
                "{}_appeal08",
                "vc_{}"
            );
            check(substr.clone());
            if substr.len() > 2 {
                for i in 0..10 {
                    check(format!("vc_{}{:02}", &substr[..substr.len() - 2], i));
                    check(format!("seq_{}{:02}", &substr[..substr.len() - 2], i));
                }
            }
            if let Some(c) = character {
                check(format!("seq_{}_rnd_{}", c, substr));
                for i in 1..substr.len() {
                    check(format!("seq_{}_rnd_{}", c, &substr[..substr.len() - i]));
                    check(format!("vc_{}_rnd_{}", c, &substr[..substr.len() - i]));
                    check(format!("seq_{}_{}", c, &substr[..substr.len() - i]));
                    check(format!("vc_{}_{}", c, &substr[..substr.len() - i]));
                }
            }
        }
    }

    let mut found: Vec<_> = just_cracked.into_iter().collect();
    found.sort();
    found.iter().for_each(|s| println!("{}", s));
}
