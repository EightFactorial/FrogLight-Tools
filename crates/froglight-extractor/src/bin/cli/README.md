# FrogLight Extractor

A simple cli tool to extract data from a Minecraft jar file.

## Usage

See `--help` for more information

From the `FrogLight` repository, you can run the tool with the following command:

```sh
just tools extract
```
For example, to extract all supported data from the jar file, you can run:
```sh
just tools extract 1.20.2
```
To search for all classes that contain the word "Block", you can run:
```sh
just tools extract 1.20.2 search Block
```

## Examples

```sh
# Extract all supported data from the jar file
froglight-extractor --cache ./target --version 1.20 extract

# Extract all supported data and save it to a file
froglight-extractor --cache ./target --version 1.20 --output ./output.json extract 

# Extract only the block states from the jar file
froglight-extractor --cache ./target --version 1.20.4-rc1 extract --module block-states
 
# Print the data of a specific class
froglight-extractor --cache ./target --version 1.20.2 print net/minecraft/block/Block.class

# Print the data of a specific class and save it to a file
froglight-extractor --cache ./target --version 1.20.2 --output ./output.json print net/minecraft/block/Block.class

# Search for all classes that contain the word "Block" (case sensitive)
froglight-extractor --cache ./target --version 1.20.1 search Block

# Search for all classes that contain the word "Air" and open the result in a text editor
froglight-extractor --cache ./target --version 1.20.1 search Air | subl -
```

> [!Note]
> If you are running this from the cargo workspace, you can use the following command to run the tool:
>
> ```sh
> # Replace { args } with the arguments you want to pass to the tool
> cargo run --package froglight-extractor --features=binary -- { args }
> ```
