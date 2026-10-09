package androidx.compose.foundation.layout

import androidx.compose.ui.Modifier

// An extension used on `Modifier`: for a server that cannot resolve
// `Modifier`, the import of it looks unused.
fun Modifier.padding(all: Int): Modifier = this
