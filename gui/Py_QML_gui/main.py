# This Python file uses the following encoding: utf-8
from __future__ import annotations
import sys
import json
from pathlib import Path

from PySide6.QtWidgets import QApplication  # Changed from QGuiApplication to QApplication
from PySide6.QtQml import QQmlApplicationEngine #access to qml

from PySide6.QtWidgets import (
    QWidget,
    QApplication,
    QMessageBox,
    QLineEdit,
    QProgressBar,
    QPushButton,
    QHBoxLayout,
    QVBoxLayout,
    QStyle,
    QFileDialog,
)
from PySide6.QtCore import QStandardPaths, QUrl, QFile, QSaveFile, QDir, QIODevice, Slot, QTimer
from PySide6.QtNetwork import QNetworkReply, QNetworkRequest, QNetworkAccessManager
from scipy import integrate, interpolate
import numpy as np

import pyqtgraph as pg
from PySide6.QtCore import QObject, Signal, Property


class DataGui(QObject):

    def __init__(self, clientid=0):
        super().__init__()
        self._energy = 0.0
        self._totalenergy = 0.0
        self._clientid = clientid

    energyChanged = Signal()
    totalEnergyChanged = Signal()
    clientIdChanged = Signal()
    # need getters and setters for each element for qml to use it


    def getEnergy(self):
            return self._energy

    def setEnergy(self, value):
            if self._energy != value:
                self._energy = value
                self.energyChanged.emit()

    energy = Property(float, getEnergy, setEnergy, notify=energyChanged)

    def getTotalEnergy(self):
            return self._totalenergy

    def setTotalEnergy(self, value):
        if self._totalenergy != value:
                self._totalenergy = value
                self.totalEnergyChanged.emit()

    totalenergy = Property(float, getTotalEnergy, setTotalEnergy, notify=totalEnergyChanged)

    def getClientId(self):
            return self._clientid

    def setClientId(self, value):
        if self._clientid != value:
                self._clientid = value
                self.clientIdChanged.emit()

    clientid = Property(int, getClientId, setClientId, notify=clientIdChanged)


class Netstuff(QObject):
    def __init__(self, Data):
        super().__init__()
        #need to create instances here for acting upon later
        self.nam = QNetworkAccessManager(self)
        self.nam.finished.connect(self.processReply) #means connect finished signal of net manager calls the processReply
        #get handles and assign the windows to the plots
        self.Data = Data;
        self.win1 = pg.GraphicsLayoutWidget(title="Client 1 Voltage (mV) vs Time (ms)")
        self.win2 = pg.GraphicsLayoutWidget(title="Client 2 Voltage (mV) vs Time (ms)")
        self.win3 = pg.GraphicsLayoutWidget(title="Client 3 Voltage (mV) vs Time (ms)")
        self.win4 = pg.GraphicsLayoutWidget(title="Client 4 Voltage (mV) vs Time (ms)")
        self.plot1 = self.win1.addPlot(title="Client 1 Voltage vs Time")
        self.plot2 = self.win2.addPlot(title="Client 2 Voltage vs Time")
        self.plot3 = self.win3.addPlot(title="Client 3 Voltage vs Time")
        self.plot4 = self.win4.addPlot(title="Client 4 Voltage vs Time")
        #Show windows
        self.win1.show()
        self.win2.show()
        self.win3.show()
        self.win4.show()

    @Slot()
    def calcPowerToaster(input: float) -> float:
        voltage= input/1000.0
        resistance = 60.804
        return round(((voltage ** 2)/resistance), 4)


    @Slot()
    def sendRequest(self):
        url = QUrl("http://192.168.2.1:8080")
        request = QNetworkRequest(url)
        self.reply = self.nam.get(request) #calls get on the selfs request variable (defined above)

    @Slot()
    def processReply(self, reply: QNetworkReply): #two arguments, needs to recieve reply
        er = reply.error()
        if er == QNetworkReply.NoError:
            answerAsText = bytes(reply.readAll()).decode("utf-8") #decode to text for turning into graph

            readings = json.loads(answerAsText)
            print(readings)

            #here i want to put the values i get from readings into the plots declared above
            data1 = readings["client1"].get("readings", [])
            times1 = [item["sensor_time"] for item in data1]
            values1 = [item["sensor_value"] for item in data1]
            power_values1 = [self.calcPowerToaster(v) for v in values1]
            id1 = [item["sensor_id"] for item in data1]

            energy1 = integrate.simpson(power_values1, x=times1)

            self.Data["client1"].energy = energy1
            self.Data["client1"].totalenergy = self.Data["client1"].totalenergy + energy1
            self.Data["client1"].clientid = id1[0]

            xnew = np.linspace(min(times1), max(times1), 100) #creates denser set of points for interp.
            spl = interpolate.make_interp_spline(times1, values1, 3) #cubic spline
            ynew = spl(xnew)

            self.plot1.clear()  # clear previous data
            self.plot1.plot(xnew, ynew, pen='r', symbol='o')
            #text_item1 = pg.TextItem(text=f"Current Energy: {energy1:.2f} J\nTotal Energy: {self.totalenergy1:.2f}", color='w', anchor=(0, 0)) #energy label
            #self.plot1.addItem(text_item1)
            #text_item1.setPos(30,20)


            data2 = readings["client2"].get("readings", [])
            times2 = [item["sensor_time"] for item in data2]
            values2 = [item["sensor_value"] for item in data2]
            power_values2 = [self.calcPowerToaster(v) for v in values2]
            id2 = [item["sensor_id"] for item in data2]

            energy2 = integrate.simpson(power_values2, x=times2)

            self.Data["client2"].energy = energy2
            self.Data["client2"].totalenergy = self.Data["client2"].totalenergy + energy2
            self.Data["client2"].clientid = id2[0]

            xnew = np.linspace(min(times2), max(times2), 100)  # creates a denser set of points for interpolation
            spl = interpolate.make_interp_spline(times2, values2, 3)  # cubic spline interpolation
            ynew = spl(xnew)

            self.plot2.clear()  # clear previous data
            self.plot2.plot(xnew, ynew, pen='r', symbol='o')


            data3 = readings["client3"].get("readings", [])
            times3 = [item["sensor_time"] for item in data3]
            values3 = [item["sensor_value"] for item in data3]
            power_values3 = [self.calcPowerToaster(v) for v in values3]
            id3 = [item["sensor_id"] for item in data3]

            energy3 = integrate.simpson(power_values3, x=times3)

            self.Data["client3"].energy = energy3
            self.Data["client3"].totalenergy = self.Data["client3"].totalenergy + energy3
            self.Data["client3"].clientid = id3[0]

            xnew = np.linspace(min(times3), max(times3), 100)  # creates a denser set of points for interpolation
            spl = interpolate.make_interp_spline(times3, values3, 3)  # cubic spline interpolation
            ynew = spl(xnew)

            self.plot3.clear()  # clear previous data
            self.plot3.plot(xnew, ynew, pen='r', symbol='o')


            data4 = readings["client4"].get("readings", [])
            times4 = [item["sensor_time"] for item in data4]
            values4 = [item["sensor_value"] for item in data4]
            power_values4 = [self.calcPowerToaster(v) for v in values4]
            id4 = [item["sensor_id"] for item in data4]

            energy4 = integrate.simpson(power_values4, x=times4)

            self.Data["client4"].energy = energy4
            self.Data["client4"].totalenergy = self.Data["client4"].totalenergy + energy4
            self.Data["client4"].clientid = id4[0]

            xnew = np.linspace(min(times4), max(times4), 100)  # creates a denser set of points for interpolation
            spl = interpolate.make_interp_spline(times4, values4, 3)  # cubic spline interpolation
            ynew = spl(xnew)

            self.plot4.clear()  # clear previous data
            self.plot4.plot(xnew, ynew, pen='r', symbol='o')


        else:
            print("Error occured: ", er)
            print(reply.errorString())
        reply.deleteLater() #cleans up resources


if __name__ == "__main__":
    app = QApplication(sys.argv) #creates instance of qgui and passes system to it
    engine = QQmlApplicationEngine()

    Data1 = DataGui(clientid=1)
    Data2 = DataGui(clientid=2)
    Data3 = DataGui(clientid=3)
    Data4 = DataGui(clientid=4)

    Data = { #encapsulate all data in a single dictionary
        "client1": Data1,
        "client2": Data2,
        "client3": Data3,
        "client4": Data4
    }

    print("Data dictionary:", Data)

    engine.rootContext().setContextProperty("data1", Data1) #expose to frontend
    engine.rootContext().setContextProperty("data2", Data2) #expose to frontend
    engine.rootContext().setContextProperty("data3", Data3) #expose to frontend
    engine.rootContext().setContextProperty("data4", Data4) #expose to frontend

    netstuff = Netstuff(Data) #takes data as an argument so it can be set in the processreply
    timer = QTimer() #initialse timer for repeated network requests
    timer.timeout.connect(netstuff.sendRequest) #pipes the timeout signal to send request (and therefore process reply)
    timer.start(1000) #timer repeats every 500ms



    #engine.rootContext().setContextProperty("backend", backend) #expose backend to qml


    qml_file = Path(__file__).resolve().parent / "main.qml" # create engine instance and load file to engine object
    engine.load(qml_file)
    if not engine.rootObjects(): #checks if load is succesful
        sys.exit(-1)
    sys.exit(app.exec())
