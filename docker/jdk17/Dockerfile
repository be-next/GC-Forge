# GC-Forge runner image — embeds the workload harness on Eclipse
# Temurin 17. Sibling of docker/jdk21/Dockerfile (which targets
# Temurin 21). Build context is the repository root:
#
#     docker build -f docker/jdk17/Dockerfile -t gc-forge-runner:dev-jdk17 .
#
# The Makefile target `make docker-image JDK_MAJOR=17` wraps this
# invocation. Used in particular to produce the JDK-17 half of the
# learning corpus consumed by GC-Insight.

FROM eclipse-temurin:22-jdk-jammy

LABEL org.opencontainers.image.title="gc-forge-runner"
LABEL org.opencontainers.image.description="GC-Forge workload harness on Eclipse Temurin 17."
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.source="https://github.com/jerome-ramette/gc-forge"

COPY workload-harness/target/workload-harness.jar /opt/gc-forge/harness.jar

WORKDIR /work

ENTRYPOINT ["java"]
CMD ["-Xlog:gc*=info:file=/work/gc.log:time,level,tags:filecount=0", \
     "-jar", "/opt/gc-forge/harness.jar"]
