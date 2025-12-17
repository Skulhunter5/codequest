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
