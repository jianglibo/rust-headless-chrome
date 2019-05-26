// struct Parent {
//     count: u32,
// }

// struct Child<'a> {
//     parent: &'a Parent,
// }

// struct Combined<'a> {
//     parent: Parent,
//     child: Child<'a>,
// }

// impl<'a> Combined<'a> {
//     fn new() -> Self {
//         let parent = Parent { count: 42 };
//         let child = Child { parent: &parent };
//         Combined { parent, child }
//     }
// }

#[derive(Debug)]
struct WhatAboutThis<'a> {
    name: String,
    nickname: Option<&'a str>,
}



/// https://stackoverflow.com/questions/32300132/why-cant-i-store-a-value-and-a-reference-to-that-value-in-the-same-struct
#[test]
fn t_lf_1() {
    // let _ = Combined::new();
    let mut tricky = WhatAboutThis {
        name: "Annabelle".to_string(),
        nickname: None,
    };
    tricky.nickname = Some(&tricky.name[..4]);
    println!("{:?}", tricky);
}