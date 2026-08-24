# GC-Forge runner image — embeds the workload harness on Eclipse
# Temurin 21. Sibling of docker/jdk17/Dockerfile (which targets
# Temurin 17). Build context is the repository root:
#
#     docker build -f docker/jdk21/Dockerfile -t gc-forge-runner:dev-jdk21 .
#
# The Makefile target `make docker-image` (default JDK_MAJOR=21)
# wraps this invocation.

FROM eclipse-temurin:22-jdk-jammy

LABEL org.opencontainers.image.title="gc-forge-runner"
LABEL org.opencontainers.image.description="GC-Forge workload harness on Eclipse Temurin 21."
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.source="https://github.com/jerome-ramette/gc-forge"

# The fat-jar is built from workload-harness/ and copied here. The build
# context is the repository root.
COPY workload-harness/target/workload-harness.jar /opt/gc-forge/harness.jar

# Default workdir for output volume mounts.
WORKDIR /work

# The DockerRunner overrides this entirely with `-Xlog:gc*` flags + harness
# args. The default just demoes a 10 s allocation loop with verbose GC.
ENTRYPOINT ["java"]
CMD ["-Xlog:gc*=info:file=/work/gc.log:time,level,tags:filecount=0", \
     "-jar", "/opt/gc-forge/harness.jar"]
