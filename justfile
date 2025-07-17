build BEDROCK_VERSION JAVA_VERSION:
    mkdir -p dist
    cargo run -- --override-map ./override-map.json --emit-map --output dist/ auto -b {{BEDROCK_VERSION}} -j {{JAVA_VERSION}}
