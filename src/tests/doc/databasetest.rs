#[cfg(test)]
use crate::utils::misc::SpekiPaths;
use crate::utils::sql::init_db;
fn get_paths() -> SpekiPaths {
    let home = home::home_dir().unwrap();
    let mut paths = SpekiPaths::new(&home);
    paths.database = home.join("dbtest.db");
    paths
}

#[test]
fn initdbtest() {
    let paths = get_paths();
    init_db(&paths.database).unwrap();
}
