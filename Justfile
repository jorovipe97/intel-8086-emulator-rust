file := "listings_asm/listing_0041_add_sub_cmp_jnz"

run:
    @echo "Assembling original file..."
    ./nasm {{file}}.asm
    @echo "Assembling succesfull!"

    @echo "Running 8086 disassembler..."
    cargo run -- {{file}}
    @echo "Program finished correctly!"

    @echo "Assembling disassembled file..."
    ./nasm result.asm
    @echo "Assembling succesfull!"

    @echo "Comparing both binaries, if nothing appears next, they are equal"
    cmp {{file}} result

test:
    cargo test
