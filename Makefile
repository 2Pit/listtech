.PHONY: run ingest

run:
	tmux new-session -d -s listtech \
		'cargo run -p indexer' \; \
		split-window -v 'cargo run -p searcher' \; \
		select-layout even-vertical \; \
		attach

ingest:
	cargo run -p ingest --release
