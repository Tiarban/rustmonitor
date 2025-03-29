import QtQuick
import QtQuick.Window
import QtQuick.Controls
import QtQuick.Layouts

//creates the main window
Window {
    width: 640
    height: 480
    visible: true
    title: qsTr("Energy Monitor")

    ColumnLayout {
    anchors.centerIn: parent

    Text {
            text: "Client ID: " + data1.clientid + "    Current Energy: " + data1.energy + " J" + "    Total Energy: " + data1.totalenergy+ " J"
            font.pixelSize: 10
        }
    Text {
            text: "Client ID: " + data2.clientid + "    Current Energy: " + data2.energy + " J" + "    Total Energy: " + data2.totalenergy + " J"
            font.pixelSize: 10
        }
    Text {
            text: "Client ID: " + data3.clientid + "    Current Energy: " + data3.energy + " J" + "    Total Energy: " + data3.totalenergy + " J"
            font.pixelSize: 10
            }
    Text {
            text: "Client ID: " + data4.clientid + "    Current Energy: " + data4.energy + " J" + "    Total Energy: " + data4.totalenergy + " J"
            font.pixelSize: 10
            }

    }

}
