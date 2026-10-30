package repositories

import dataClasses.DiscoveredDevice
import dataClasses.DiscoverState
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import repositories.interfaces.DiscoverRepo
import uniffi.vociferate_core.ConnectionEvent
import uniffi.vociferate_core.DiscoveryEvent
import uniffi.vociferate_core.PeerInfo
import uniffi.vociferate_core.connectToPeer
import uniffi.vociferate_core.disconnectPeer
import uniffi.vociferate_core.nextConnectionEvent
import uniffi.vociferate_core.nextDiscoveryEvent
import uniffi.vociferate_core.startServer

class DiscoverRepoImpl(
    private val scope: CoroutineScope,
) : DiscoverRepo {


    private val defaultServerAddress = "0.0.0.0"
    private val defaultServerPort = "19090"

    override suspend fun startDiscovery(){
        mutableState.update { it.copy(devicesList = emptyMap(), isDiscovering = false, errorMessage = null) }

        val result = runCatching { uniffi.vociferate_core.startDiscovery() }
        result.fold(
            onSuccess = { _ ->

            },
            onFailure = { failure ->
                mutableState.update {
                    it.copy(
                        isDiscovering = false,
                        errorMessage = failure.message,
                    )
                }
            },
        )
    }

    init {
        scope.launch {
            withContext(Dispatchers.IO){
                // реактивный список доступных устройств
                launch {
                    while (isActive) {
                        val event = nextDiscoveryEvent() ?: break

                        mutableState.update { state ->
                            val currentMap = state.devicesList.toMutableMap()
                            when (event) {
                                is DiscoveryEvent.PeersAdded,
                                is DiscoveryEvent.PeersUpdated -> {
                                    val peerList = when (event) {
                                        is DiscoveryEvent.PeersAdded -> event.v1
                                        is DiscoveryEvent.PeersUpdated -> event.v1
                                        else -> emptyList()
                                    }

                                    for (peer in peerList) {
                                        val device = peer.toDevice()
                                        val existing = currentMap[device.id]
                                        currentMap[device.id] = device.copy(
                                            isSelected = existing?.isSelected ?: false
                                        )
                                    }
                                }
                                is DiscoveryEvent.PeersRemoved -> {
                                    for (id in event.v1) {
                                        currentMap.remove(id)
                                    }
                                }
                            }
                            state.copy(devicesList = currentMap)
                        }
                    }
                }

                // тут прилетает метка активности соединения для устройства в списке обнаруженных
                launch {
                    while (isActive) {
                        val event = nextConnectionEvent() ?: break
                        mutableState.update { state ->
                            val currentMap = state.devicesList.toMutableMap()
                            when (event) {
                                is ConnectionEvent.Connected -> {
                                    currentMap[event.v1.id]?.let { device ->
                                        currentMap[event.v1.id] = device.copy(isSelected = true)
                                    }
                                }
                                is ConnectionEvent.Disconnected -> {
                                    currentMap[event.v1]?.let { device ->
                                        currentMap[event.v1] = device.copy(isSelected = false)
                                    }
                                }
                                is ConnectionEvent.Error -> {
                                    println("ошибка соединения")
                                }
                                is ConnectionEvent.AudioData -> {
                                    // Если у тебя в AudioData прилетает инфа или ты хочешь как-то реагировать
                                }
                            }
                            state.copy(devicesList = currentMap)
                        }
                    }
                }
            }
        }
    }

    private val mutableState = MutableStateFlow(DiscoverState())

    override val state: StateFlow<DiscoverState> = mutableState.asStateFlow()

    override suspend fun startServerKotlin() {
        withContext(Dispatchers.IO){
            startServer("$defaultServerAddress:$defaultServerPort")
        }
    }

    override suspend fun connectToPeer(device: DiscoveredDevice) {
        withContext(Dispatchers.IO){
            val peerInfo = PeerInfo(
                id = device.id,
                name = device.id,
                ip = device.address,
                port = defaultServerPort.toUShort(),
                hostname = device.name,
                isActiveMicro = device.sayingSomething
            )
            connectToPeer(peerInfo)
        }
    }

    override suspend fun disconnectPeer(device: DiscoveredDevice) {
        withContext(Dispatchers.IO){
            disconnectPeer(device.id)
        }
    }
}
private fun PeerInfo.toDevice(): DiscoveredDevice = DiscoveredDevice(
    id = id,
    name = name,
    address = ip,
    port = port.toInt(),
    isSelected = false,
    sayingSomething = isActiveMicro
)