all:

docker_build:
	docker build -t codequest-bootstrap -f bootstrap/Dockerfile .
	docker build -t codequest-gateway -f gateway/Dockerfile .
	docker build -t codequest-user-service -f user-service/Dockerfile .
	docker build -t codequest-quest-service -f quest-service/Dockerfile .
	docker build -t codequest-progression-service -f progression-service/Dockerfile .

single_dockerfile_test:
	docker build .
	docker tag $$(docker image ls -a -q --filter=dangling=true --filter=label=service=bootstrap) codequest-bootstrap || true
	docker tag $$(docker image ls -a -q --filter=dangling=true --filter=label=service=gateway) codequest-gateway || true
	docker tag $$(docker image ls -a -q --filter=dangling=true --filter=label=service=users) codequest-user-service || true
	docker tag $$(docker image ls -a -q --filter=dangling=true --filter=label=service=quests) codequest-quest-service || true
	docker tag $$(docker image ls -a -q --filter=dangling=true --filter=label=service=progression) codequest-progression-service || true

GENERATORS_DIR=run/quests/generators

debug_quests:
	mkdir -p $(GENERATORS_DIR)
	gcc debug-quests/quest-1.c -o $(GENERATORS_DIR)/47ef64ab-5a84-4c4c-bed8-75086535fba3
	gcc debug-quests/quest-2.c -o $(GENERATORS_DIR)/400d5f46-9997-4da0-8703-050c504174af
	gcc debug-quests/quest-3.c -o $(GENERATORS_DIR)/e2225bb3-07b5-4005-8f0a-c393b972e988
	gcc debug-quests/quest-4.c -o $(GENERATORS_DIR)/485ff7db-b0b0-447d-80d3-099044bcd120
	gcc debug-quests/quest-5.c -o $(GENERATORS_DIR)/362f018f-7d36-40e1-9534-a0966cd81207
	gcc debug-quests/quest-6.c -o $(GENERATORS_DIR)/f1232b43-07af-4c5f-baa0-21da5a43fc83
	gcc debug-quests/quest-7.c -o $(GENERATORS_DIR)/82bdf583-2c0f-4d67-be79-2866c4a986e3
	gcc debug-quests/quest-8.c -o $(GENERATORS_DIR)/75ad32aa-76c6-4d74-a545-e9b95b48e21a

example_quests:
	cd example-quests && cargo build --release --bin quest-1 --bin quest-2 --bin quest-3 --bin quest-4 --bin quest-5
	mkdir -p $(GENERATORS_DIR)
	mv example-quests/target/release/quest-1 $(GENERATORS_DIR)/e4f16655-0dd2-4e40-b3f5-375305d848ff
	mv example-quests/target/release/quest-2 $(GENERATORS_DIR)/d2af9af8-471a-4b7e-98d8-164475908514
	mv example-quests/target/release/quest-3 $(GENERATORS_DIR)/81020ed8-9474-4eba-8256-86af495c269f
	mv example-quests/target/release/quest-4 $(GENERATORS_DIR)/ad31793a-1ba3-43fc-b738-225311b7eca6
	mv example-quests/target/release/quest-5 $(GENERATORS_DIR)/d45016cb-fc53-45fd-ae7f-8c44acabf3d8
