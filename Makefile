.PHONY: run ingest add_schema select

run:
	tmux new-session -d -s listtech \
		'cargo run -p indexer' \; \
		split-window -v 'cargo run -p searcher' \; \
		select-layout even-vertical \; \
		attach

ingest:
	cargo run -p ingest --release

add_schema:
	bash scripts/add_schema.sh

select:
	bash scripts/select.sh
