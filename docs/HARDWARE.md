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
        IN1[IN1]
        IN2[IN2]
        PWM[PWM]
        GND_D[GND]
        VCC[VCC - Logic]
        VIN[VIN - 12V]
        M1+[Output 1+]
        M1-[Output 1-]
    end

    subgraph Linear Actuator
        LA_M+[Motor +]
        LA_M-[Motor -]
        LA_VCC[Encoder VCC]
        LA_GND[Encoder GND]
        LA_A[Phase A]
        LA_B[Phase B]
    end

    subgraph PSU [12V Power Supply]
        P12[+12V]
        PGND[GND]
    end

    %% Arduino to Driver Logic
    D5 --> IN1
    D6 --> IN2
    D9 --> PWM
    V5 --> VCC
    GND_A --> GND_D

    %% Power to Driver
    P12 --> VIN
    PGND --> GND_D

    %% Driver to Actuator Motor
    M1+ --> LA_M+
    M1- --> LA_M-

    %% Actuator Encoder to Arduino
    LA_VCC --> V5
    LA_GND --> GND_A
    LA_A --> D2
    LA_B --> D3

    %% Global GND Bond
    PGND --- GND_A
```
