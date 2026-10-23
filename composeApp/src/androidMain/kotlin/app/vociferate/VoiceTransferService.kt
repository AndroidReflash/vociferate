package app.vociferate

import android.R
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import androidx.core.app.ServiceCompat
import app.maps.constants.ServiceConstantsInts
import app.maps.constants.ServiceConstantsStrings

class VoiceTransferService: Service() {
    override fun onCreate() {
        super.onCreate()
        val manager = getSystemService(NotificationManager::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            manager?.createNotificationChannel(
                NotificationChannel(
                    ServiceConstantsStrings.ChannelId.value,
                    ServiceConstantsStrings.ChannelName.value,
                    NotificationManager.IMPORTANCE_LOW,
                )
            )
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        ServiceCompat.startForeground(
            this,
            ServiceConstantsInts.NotificationId.value,
            notification(),
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                ServiceInfo.FOREGROUND_SERVICE_TYPE_MICROPHONE
            } else {
                0
            },
        )
        println("askldqow")
        return START_NOT_STICKY
    }

    override fun onDestroy() {
        super.onDestroy()
        println("сервис всё")
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private fun notification(): Notification =
        NotificationCompat.Builder(this, ServiceConstantsStrings.ChannelId.value)
            .setContentTitle(ServiceConstantsStrings.NotificationTitle.value)
            .setContentText(ServiceConstantsStrings.NotificationText.value)
            .setSmallIcon(R.drawable.ic_menu_mylocation)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .setOngoing(true)
            .build()
}