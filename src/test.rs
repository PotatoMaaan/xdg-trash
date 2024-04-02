use crate::UnifiedTrash;

#[test]
fn test_list() {
    microlog::init(log::LevelFilter::Trace);
    let trash = UnifiedTrash::new().unwrap();

    let x = trash
        .list()
        .inspect(|x| println!("{}", x.display()))
        .collect::<Vec<_>>();
}
