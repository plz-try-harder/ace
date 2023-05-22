# ace
A software that detects wildfires and filters them out of the image passed into it, creating another image with only the areas of the fire. Other areas are cut out and replaced with black Pixels.

# How to build the software:
1. You must be in the "ace/" directory first. using this command 
$ cd ace/
2. Now run this command:
$ cargo build --color=always --message-format=json-diagnostic-rendered-ansi --package ace_proj --bin ace_proj

# How to run the software:
1. You must be in the "ace/" directory first. using this command
$ cd ace/
2. Now run this command:
$ cargo run --color=always --package ace_proj --bin ace_proj

# How to pass an image into the software:
1. open the file "ace/src/main.rs" an on the line that has the line 
let file_name: &str = "test5.jpg";
Change "test5.jpg" to the name of the file/image you want to pass.
2. The image must be in the "ace/" directory.
