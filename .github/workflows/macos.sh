version=$(grep "^version" Cargo.toml | cut -d"\"" -f2 )
if [[ $version != *"-SNAPSHOT"* ]]; then
  echo "Version string MUST contain \"-SNAPSHOT\"!"
  exit 1;
fi
version=$(echo $version | sed 's/-SNAPSHOT//g')
tag=$(echo ${GITHUB_REF/refs\/tags\//} | sed 's/^.//')
if [[ $version != $tag ]]; then
  echo "Version Mismatch! The version you want to build doesn't match the version in your Cargo.toml.";
  exit 1;
fi
sed -i '' "s/^version.*$/version = \"$version\"/g" Cargo.toml