use crate::UnifiedTrash;

#[test]
fn test_list() {
    microlog::init(log::LevelFilter::Trace);
    let trash = UnifiedTrash::new().unwrap();

    let x = trash
        .list()
        .map(|x| x.unwrap())
        .inspect(|x| println!("{:#?}", x))
        .collect::<Vec<_>>();
}
