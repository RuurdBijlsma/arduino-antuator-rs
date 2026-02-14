#define RPWM 5
#define LPWM 6

void setup() {
  pinMode(RPWM, OUTPUT);
  pinMode(LPWM, OUTPUT);

  digitalWrite(LPWM, LOW); // backward off
}

void loop() {
  analogWrite(RPWM, 200); // forward at ~80%
  delay(5000);
  analogWrite(RPWM, 0);   // stop
  delay(2000000);
}
