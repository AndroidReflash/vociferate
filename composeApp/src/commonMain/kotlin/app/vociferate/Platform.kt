package app.vociferate

interface Platform {
    val name: String
}

expect fun getPlatform(): Platform
