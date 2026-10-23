package repositories

import dataClasses.StartScreenState
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import repositories.interfaces.StartScreenRepo
import uniffi.vociferate_core.getDeviceName
import uniffi.vociferate_core.setDeviceName

class StartScreenRepoImpl(
    private val scope: CoroutineScope,
) : StartScreenRepo {

    private val mutableState = MutableStateFlow(StartScreenState())

    override val state: StateFlow<StartScreenState> = mutableState.asStateFlow()

    private val _deviceName = MutableStateFlow<String?>(null)
    override val deviceName : StateFlow<String?> = _deviceName

    override fun loadDeviceName() {
        _deviceName.value = getDeviceName()
    }

    override fun saveDeviceName(name: String) {
        val trimmed = name.trim()
        if (trimmed.isBlank()) {
            mutableState.update { it.copy(errorMessage = "Имя не может быть пустым") }
            return
        }
        scope.launch(Dispatchers.IO) {
            setDeviceName(trimmed)
        }
        _deviceName.value = name
    }
}
