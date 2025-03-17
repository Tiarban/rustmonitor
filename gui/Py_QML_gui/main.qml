import QtQuick
import QtQuick.Window
import QtQuick.Controls
import QtQuick.Layouts

//creates the main window
Window {
    width: 640
    height: 480
    visible: true
    title: qsTr("Hello World")

    readonly property list<string> texts: ["Hallo Welt", "Hei maailma",
                                           "Hola Mundo", "Привет мир"]

    function setText () {
        var i = Math.round(Math.random()*3)
        text.text = texts[i]
    }

    ColumnLayout {
    anchors.fill: parent

    Text {
        id: text
        text: "yuppa"
        Layout.alignment: Qt.AlignHCenter
    }

    Button {
        text: "Click me"
        Layout.alignment: Qt.AlignHCenter
        onClicked: setText()
    }

    RowLayout {
    Layout.fillWidth: true
    spacing: 10

    Text {
        id: firstclientstatus
        text: "Status: Disconnected"
        Layout.alignment: Qt.AlignLeft
    }

    Text {
        id: secondclientstatus
        text: "Status: Disconnected"
        Layout.alignment: Qt.AlignLeft
    }


    Button {
        text: "Show Client 1"
        Layout.alignment: Qt.AlignRight
        Layout.margins: 20
        onClicked: {
            backend.showGraph()
        }
    }

    Button {
        text: "Show Client 2"
        Layout.alignment: Qt.AlignRight
        Layout.margins: 20
        onClicked: {
            backend.showGraph()
        }
    }

    Button {
        text: "Show Client 3"
        Layout.alignment: Qt.AlignRight
        Layout.margins: 20
        onClicked: {
            backend.showGraph()
        }
    }

    Button {
        text: "Show Client 4"
        Layout.alignment: Qt.AlignRight
        Layout.margins: 20
        onClicked: {
            backend.showGraph()
        }
    }

    }

    }

}
