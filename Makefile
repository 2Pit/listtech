.PHONY: swagger-indexer swagger-searcher run ingest

swagger-indexer:
	./scripts/update-swagger.sh indexer/proto/api.proto corelib/static/swagger-ui/indexer

swagger-searcher:
	./scripts/update-swagger.sh searcher/proto/api.proto corelib/static/swagger-ui/searcher

run:
	tmux new-session -d -s listtech \
		'cargo run -p indexer' \; \
		split-window -v 'cargo run -p searcher' \; \
		select-layout even-vertical \; \
		attach

ingest:
	cargo run -p ingest
