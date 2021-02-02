// use rl_ecs::{RlEcs, RlEcsBuilder, BinStorage};
// use rl_ecs::{Item, Parent, Target};
// use rl_ecs::EntityRelationType;
// use rl_ecs::ids::*;
//
// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Entity {}
// impl Item for Entity { type ID=ID0; }
//
// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Desc {
// name: String,
// icon: usize,
// }
// impl Item for Desc { type ID=ID1; const TYPE: EntityRelationType =
// EntityRelationType::Flag; }
//
// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Pos {
// x: i32,
// y: i32,
// }
// impl Item for Pos { type ID=ID2; const TYPE: EntityRelationType =
// EntityRelationType::Flag; }
//
// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct Node {}
// impl Item for Node { type ID=ID3; const TYPE: EntityRelationType =
// EntityRelationType::UniqueChildren; }
//
// #[test]
// fn test_create() {
// let _ = simple_logger::init();
//
// initialise ecs
// let mut ecs = <(Entity, Desc, Pos, Node)>::create_ecs();
// }
//
// #[test]
// fn test_add() {
// let _ = simple_logger::init();
//
// initialise ecs
// let mut ecs = <(Entity, Desc, Pos, Node)>::create_ecs();
//
// spawn first item, and purge
// let p = ecs.create(Pos { x: 1, y: 2 });
// assert_eq!(ecs.id.contains(p), true);
// ecs.purge(p);
// assert_eq!(ecs.id.contains(p), false);
// }
//
// #[test]
// fn test_simple() {
// let _ = simple_logger::init();
//
// initialise ecs
// let mut ecs = <(Entity, Desc, Pos, Node)>::create_ecs();
//
// add second item
// let e1 = ecs.create(Entity {});
// assert!(ecs.id.contains(e1));
//
// create a description, attach to entity and do some checks
// let desc1 = ecs.create(Desc { name: "Orc".to_string(), icon: 1 });
// assert!(ecs.attach(desc1, e1));
// assert!(ecs.contains(desc1));
// assert!(ecs.get_child_id::<Desc>(e1).is_some() );
// assert_eq!(ecs.get_parent_id(desc1), Some(e1));
// assert_eq!(ecs.get::<Desc>(desc1).unwrap().name, "Orc".to_string());
//
// create and attach a position to our first item
// assert!(!ecs.get_child_id::<Pos>(e1).is_some() );
// let pos1 = ecs.create_and_attach(Pos { x: 3, y: 4 }, e1).unwrap();
//
// check if pos1 exists via it's own id
// assert!(ecs.contains(pos1));
// check if pos1 exists via the parent id
// assert!(ecs.get_child_id::<Pos>(e1).is_some() );
// assert_eq!(ecs.get_child_id::<Pos>(e1), Some(pos1));
// assert_eq!(ecs.get_parent_id(pos1), Some(e1));
//
// ensure pos1 has the correct data
// assert_eq!(ecs.get::<Pos>(pos1).unwrap(), &Pos { x: 3, y: 4 });
//
// spawn a second item, and ensure it is different than the first
// let e2 = ecs.create(Entity {});
// assert!(ecs.contains(e2));
// assert_ne!(e2, e1);
//
// attach some children to e2
// assert_ne!(ecs.create_and_attach(Pos { x: 5, y: 5 }, e2), None);
// assert_ne!(ecs.create_and_attach(Desc { name: "Dwarf".to_string(), icon: 9 },
// e2), None); assert_eq!((ecs.get::<Pos>(e2).unwrap()).x, 5);
// assert_eq!((ecs.get::<Desc>(e2).unwrap()).icon, 9);
//
// assert!(ecs.contains(e1));
// assert!(ecs.detach(desc1));
// assert!(!ecs.get_child_id::<Desc>(e1).is_some() );
// assert!(ecs.contains(desc1));
// assert!(ecs.purge(desc1));
// assert_eq!(ecs.contains(desc1), false);
// remove e1 and it's children, and ensure e2 still exists
// assert!(ecs.purge(e1));
// assert!(!ecs.get_child_id::<Pos>(e1).is_some() );
// assert!(!ecs.contains(e1));
// assert!(ecs.get_child_id::<Pos>(e2).is_some() );
// assert!(ecs.get_child_id::<Desc>(e2).is_some() );
// }
//
// #[test]
// fn test_advanced() {
// let _ = simple_logger::init();
//
// initialise ecs
// let mut ecs = <(Entity, Desc, Pos, Node)>::create_ecs();
//
// spawn a second item, and ensure it is different than the first
// let e1 = ecs.create(Entity {});
// assert!(ecs.contains(e1));
//
// attach some children to e1
// assert_ne!(ecs.create_and_attach(Pos { x: 5, y: 5 }, e1), None);
// assert_ne!(ecs.create_and_attach(Desc { name: "Dwarf".to_string(), icon: 9 },
// e1), None); assert_eq!((ecs.get::<Pos>(e1).unwrap()).x, 5);
// assert_eq!((ecs.get::<Desc>(e1).unwrap()).icon, 9);
//
// spawn another and give it children and grand children
// let e2 = ecs.create(Entity {});
// let node_e2 = ecs.create_and_attach(Node {}, e2).unwrap();
// assert!(ecs.id.contains(node_e2));
// let desc_e2 = ecs.create_and_attach(Desc { name: "Room".to_string(), icon: 2
// }, node_e2).unwrap(); assert!(ecs.contains(desc_e2));
// assert!(ecs.contains(e2));
// assert!(ecs.contains(e2));
// assert!(ecs.contains(node_e2));
//
// remove e3 and check e2
// assert!(ecs.purge(e2));
// assert_eq!((ecs.get::<Desc>(e1).unwrap()).icon, 9);
// assert_eq!((ecs.get::<Pos>(e1).unwrap()).x, 5);
//
// Everything should be empty
// assert!(ecs.purge(e1));
// assert_eq!(ecs.contains(e1), false);
// assert_eq!(ecs.contains(e2), false);
// assert_eq!(ecs.contains(node_e2), false);
// assert_eq!(ecs.contains(desc_e2), false);
//
// }
//
// #[test]
// fn test_matcher() {
// let _ = simple_logger::init();
//
// initialise ecs
// let mut ecs = <(Entity, Desc, Pos, Node)>::create_ecs();
//
// let e4 = ecs.create(Entity {});
// ecs.create_and_attach(Desc { name: "Test".to_string(), icon: 2 }, e4);
// ecs.create_and_attach(Pos { x: 5, y: 5 }, e4);
// ecs.purge(e4);
//
// for _ in ecs.bin.iter::<Entity>() {
// assert!(false);
// }
// for _ in ecs.bin.iter_mut::<Entity>() {
// assert!(false);
// }
//
// let e = ecs.create(Entity {});
// let p = ecs.create_and_attach(Pos { x: 5, y: 5 }, e).unwrap();
// for (id,) in ecs.id.matcher::<(Target<Pos>,)>() {
// let p2 = ecs.bin.get_mut::<Pos>(id).unwrap();
// p2.x = 6;
// };
// for (p, _) in ecs.id.matcher::<(Target<Pos>, Parent<Entity>)>() {
// let p2 = ecs.bin.get_mut::<Pos>(p).unwrap();
// p2.x = 6;
// }
// assert_eq!(ecs.bin.get::<Pos>(p).unwrap().x, 6);
// }
