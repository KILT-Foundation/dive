BUILD_IMAGE=dive/builder:latest

DEPLOY_TARGET_HOST=pi@10.10.0.2 
 
default: debug-build

#Create a debug build
debug-build: .build-image $(shell find ./src -type f)
	docker run --rm -it \
		-v $(shell pwd):/app \
		-w /app \
		$(BUILD_IMAGE) \
		cargo build --target=aarch64-unknown-linux-gnu $(BUILD_ARGS)

#Create a release build
# thats buggy. make sure the build image is created before running this. 
release-build: .build-image $(shell find ./src -type f)
	docker run --rm -it \
		-v $(shell pwd):/app \
		-w /app \
		$(BUILD_IMAGE) \
		cargo build --release --target=aarch64-unknown-linux-gnu $(BUILD_ARGS)

# deploy the release binary to the target host
deploy: 
	release-build
	scp -P $(DEPLOY_TARGET_PORT) target/aarch64-unknown-linux-gnu/release/dive $(DEPLOY_TARGET_HOST):/home/pi/ssi-server

# build the build image
.build-image: 
	builder.Containerfile
	docker build -t $(BUILD_IMAGE) -f builder.Containerfile .
	

# run the debug build locally, you will need to install ca-certificates but its a nice start
run-locally: 
	debug-build
	docker run --arch arm64 -it -v .:/app -w /app debian /app/target/aarch64-unknown-linux-gnu/debug/dive
 