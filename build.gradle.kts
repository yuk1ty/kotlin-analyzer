plugins {
    kotlin("jvm") version "2.0.0"
}

group = "io.github.yuk1ty"
version = "0.1.0"

repositories {
    mavenCentral()
}

dependencies {
    implementation(libs.clikt)
    implementation(libs.lsp4j)
    implementation(libs.lsp4j.jsonrpc)
    implementation(libs.slf4j.jul)
    implementation(libs.slf4j.simple)
    implementation(libs.slf4j.api)
    implementation(libs.kotlin.logging.jvm)
    implementation(libs.kotlin.result)
    testImplementation(libs.kotlin.test)
}

tasks.test {
    useJUnitPlatform()
}
kotlin {
    jvmToolchain(17)
}