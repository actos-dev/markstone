#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Prioritize JDK 25 if present, or JAVA_HOME if set, else java on PATH
if [ -x "/usr/lib/jvm/java-25-openjdk/bin/java" ]; then
    JAVA_BIN="/usr/lib/jvm/java-25-openjdk/bin/java"
elif [ -n "$JAVA_HOME" ] && [ -x "$JAVA_HOME/bin/java" ]; then
    JAVA_BIN="$JAVA_HOME/bin/java"
else
    JAVA_BIN="java"
fi

CP="$DIR/target/classes"
if [ -f "$DIR/target/markstone-0.1.0.jar" ]; then
    CP="$DIR/target/markstone-0.1.0.jar"
fi
if [ -d "$DIR/target/dependency" ]; then
    CP="$CP:$DIR/target/dependency/*"
fi

exec "$JAVA_BIN" --enable-native-access=ALL-UNNAMED -cp "$CP" io.github.dethrandir.markstone.ConformanceRunner "$@"
