.PHONY: swagger-indexer swagger-searcher

swagger-indexer:
	./scripts/update-swagger.sh indexer/proto/api.proto corelib/static/swagger-ui/indexer

swagger-searcher:
	./scripts/update-swagger.sh searcher/proto/api.proto corelib/static/swagger-ui/searcher
