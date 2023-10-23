BUILD_IMAGE=dive/builder:latest

# ssh://pi@proxy51.rt3.io:37972
# ssh://pi@proxy55.rt3.io:34097
# ssh://pi@proxy50.rt3.io:33920
DEPLOY_TARGET_HOST=pi@proxy50.rt3.io
DEPLOY_TARGET_PORT=33920

# default target
default: debug-build

# shortcuts for building debug and release binaries
debug-build: target/aarch64-unknown-linux-gnu/debug/ssi-server
release-build: target/aarch64-unknown-linux-gnu/release/ssi-server

# debug build through the build image
target/aarch64-unknown-linux-gnu/debug/ssi-server: .build-image $(shell find ./src -type f)
	podman run --rm -it \
		-v $(shell pwd):/app \
		-w /app \
		$(BUILD_IMAGE) \
			cargo build --target=aarch64-unknown-linux-gnu

# release build through the build image
target/aarch64-unknown-linux-gnu/release/ssi-server: .build-image $(shell find ./src -type f)
	podman run --rm -it \
		-v $(shell pwd):/app \
		-w /app \
		$(BUILD_IMAGE) \
			cargo build --release --target=aarch64-unknown-linux-gnu

# deploy the release binary to the target host
deploy: release-build
	scp -P $(DEPLOY_TARGET_PORT) target/aarch64-unknown-linux-gnu/release/ssi-server $(DEPLOY_TARGET_HOST):/home/pi/ssi-server 

# build the build image
.build-image: builder.Containerfile
	podman build -t $(BUILD_IMAGE) -f builder.Containerfile .
	touch .build-image

# clean up the build artifacts
clean:
	rm -rf target
	rm -f .build-image