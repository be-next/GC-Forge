# GC-Forge runner image — embeds the workload harness on Temurin 21.
# Used by `make demo` (iter 1) and by the DockerRunner backend (iter 3).

FROM eclipse-temurin:21-jdk-jammy

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
