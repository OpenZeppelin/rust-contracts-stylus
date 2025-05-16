# Function to retrieve all Cargo.toml paths in the ./examples directory
get_example_dirs() {
  find ./examples -maxdepth 2 -type f -name "Cargo.toml" | xargs -n1 dirname | sort
}
