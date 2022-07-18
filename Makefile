play:
	bash -c "pacat <(cargo run --release)"
mac:
	bash -c "ffplay -f s16le -ar 44100 <(cargo run --release)"
