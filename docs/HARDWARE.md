# Hardware Reference

This document covers the specific hardware components and their electrical connections.

## Components

### 1. Arduino Uno

* **Role**: Main microcontroller.
* **Logic Voltage**: 5V.

### 2. DFR0601 Dual-Channel DC Motor Driver

* **Link**: [Product Page](https://www.dfrobot.com/product-1861.html).

### 3. Antuator Linear Actuator

* **Feedback**: Built-in Hall effect sensor.
* **Voltage**: 12V DC.
* **Link**: [Technical Datasheet](../docs/荷兰+Antuator+房车.pdf).

## Wiring Diagram

```mermaid
graph TD
    subgraph Arduino Uno
        D2[Pin D2 - Encoder A]
        D3[Pin D3 - Encoder B]
        D5[Pin D5 - Motor Dir 1 / IN1]
        D6[Pin D6 - Motor Dir 2 / IN2]
        D9[Pin D9 - Motor PWM]
        GND_A[GND]
        V5[5V Output]
    end

    subgraph DFR0601 Motor Driver
        PPLS[PSU +]
        PGND[PSU GND]
        M1+[Motor 1 +]
        M1-[Motor 1 GND]
        MD_V1[Pin V1]
        MD_P1[Pin P1]
        MD_A1[Pin A1]
        MD_B1[Pin B1]
        MD_G1[Pin G1]
    end

    subgraph Linear Actuator 1
        LA_M+[Motor +]
        LA_M-[Motor -]
        LA_SENS_A_VOUT[Sensor A Vout]
        LA_SENS_B_VOUT[Sensor B Vout]
        LA_SENS_GND[Sensor GND]
        LA_SENS_VCC+[Sensor VCC+]
    end

    subgraph PSU [12V Power Supply]
        P12[+12V]
        PSU_GND[GND]
    end

    %% Arduino to Driver Logic
    D5 --> MD_A1
    D6 --> MD_B1
    D9 --> MD_P1
    V5 --> MD_V1
    GND_A --> MD_G1

    %% Power to Driver
    P12 --> PPLS
    PGND --> PSU_GND

    %% Driver to Actuator Motor
    M1+ --> LA_M+
    M1- --> LA_M-

    %% Actuator Encoder to Arduino
    LA_SENS_VCC+ --> V5
    LA_SENS_GND --> GND_A
    LA_SENS_A_VOUT --> D2
    LA_SENS_B_VOUT --> D3
```
