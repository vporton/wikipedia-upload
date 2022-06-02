use walkdir::WalkDir;

fn main() {
    WalkDir::new("foo")
        .sort_by_file_name(); // keep the order deterministic, because we overwrite files
}
