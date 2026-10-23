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
        mutableState.update { it.copy(devices = emptyList(), isDiscovering = false, errorMessage = null) }

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
                            when (event) {
                                is DiscoveryEvent.PeersAdded -> {
                                    val newDevices = event.v1.map(PeerInfo::toDevice)
                                    val existingIds = newDevices.map { it.id }.toSet()
                                    val filteredCurrent = state.devices.filter { it.id !in existingIds }
                                    state.copy(devices = filteredCurrent + newDevices)
                                }
                                is DiscoveryEvent.PeersRemoved -> {
                                    val removeSet = event.v1.toSet()
                                    state.copy(devices = state.devices.filter { it.id !in removeSet })
                                }
                            }
                        }
                    }
                }
                // тут прилетает метка активности соединения для устройства в списке обнаруженных
                launch {
                    while(isActive){
                        val event = nextConnectionEvent() ?: break
                        when (event) {
                            is ConnectionEvent.Connected -> {
                                mutableState.update {
                                    it.copy(
                                        devices = it.devices.map { device ->
                                            if (device.id == event.v1.id) device.copy(isSelected = true) else device
                                        }
                                    )
                                }
                            }
                            is ConnectionEvent.Disconnected -> {
                                mutableState.update {
                                    it.copy(
                                        devices = it.devices.map { device ->
                                            if (device.id == event.v1) device.copy(isSelected = false) else device
                                        }
                                    )
                                }
                            }
                            is ConnectionEvent.Error -> {
                                println("ошибка")
                            }
                            is ConnectionEvent.AudioData -> {
                                println("звук на котлин ${event.v1}")
                            }
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
)