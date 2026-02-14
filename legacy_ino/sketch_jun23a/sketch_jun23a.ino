#include <QuadratureEncoder.h>

#define FORWARD_PIN 5
#define BACKWARD_PIN 6
#define PWM_PIN 9

Encoders encoder(2, 3);     // Create an Encoder object name leftEncoder, using digitalpin 2 & 3

// Notes:
// TODO:
// absolute waarde krijgen met 

void setup() {
  // Change this to set direction moved by actuator
  auto direction = FORWARD_PIN;
  auto moveSeconds = 5;

  // Setup
  Serial.begin(9600);
  pinMode(FORWARD_PIN, OUTPUT);
  pinMode(BACKWARD_PIN, OUTPUT);
  pinMode(PWM_PIN, OUTPUT);
  StopMoving();

  // Print pwm values before moving
  for (size_t i = 0; i < 1000; i++) {
    delay(1);
    PrintPWM();
  }

  // Move actuator 5 seconds while printing pwm values
  digitalWrite(direction, HIGH);
  analogWrite(PWM_PIN, 255);

  for (size_t i = 0; i < moveSeconds * 1000 / 2; i++) {
    delay(2);
    PrintPWM();
  }

  StopMoving();
}

void PrintPWM() {
  long encoderCount = encoder.getEncoderCount();

  Serial.println(encoderCount);
}

void StopMoving() {
  digitalWrite(FORWARD_PIN, LOW);
  digitalWrite(BACKWARD_PIN, LOW);
}

void loop() {
  PrintPWM();
  delay(100);
}