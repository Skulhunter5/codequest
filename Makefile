all:

docker_build:
	docker build -t codequest-gateway -f gateway/Dockerfile .

single_dockerfile_test:
	docker build .
	docker tag $$(docker image ls -a -q --filter=dangling=true --filter=label=service=gateway) codequest-gateway || true
	docker tag $$(docker image ls -a -q --filter=dangling=true --filter=label=service=users) codequest-user-service || true
	docker tag $$(docker image ls -a -q --filter=dangling=true --filter=label=service=quests) codequest-quest-service || true
	docker tag $$(docker image ls -a -q --filter=dangling=true --filter=label=service=progression) codequest-progression-service || true
