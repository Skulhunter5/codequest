INSERT INTO quests (id, name, description) VALUES ('e4f16655-0dd2-4e40-b3f5-375305d848ff', 'The Static Leak',
'Your first relay station reports erratic readings in its intake buffer.
Each incoming signal arrives as a short burst of measured strengths over time.
Under normal conditions, these values fluctuate gently, like currents around a reef.

Now, some of them spike.

The relay’s firmware defines a maximum safe fluctuation threshold.
Anything beyond it risks permanent desynchronization.

Input
- The first line contains a single integer T, the maximum allowed difference.
- Each subsequent line represents one signal packet.
- Each packet is a sequence of space-separated integers.

A packet is considered unstable if the absolute difference between any two consecutive values exceeds T.

Count how many packets are unstable.');

INSERT INTO quests (id, name, description) VALUES ('d2af9af8-471a-4b7e-98d8-164475908514', 'Relay Address Resolution',
'Every relay in the network identifies its neighbors using compressed numeric addresses.
The compression saves bandwidth, but decoding must be flawless. A single error, and a message vanishes into sediment.

Your station reports a backlog of unresolved addresses.
Before routing can resume, each one must be validated.

Input
- The first line contains an integer M, the checksum modulus.
- Each subsequent line contains one encoded address as a positive integer.

Decoding Rules
For each address:
1. Process digits from left to right.
2. Maintain a checksum, starting at 0.
3. For each digit:
   - If the digit is even, add it to the checksum.
   - If the digit is odd, multiply the checksum by the digit.
4. An address is valid if checksum mod M == 0.

Decode all addresses and compute the sum of the valid original addresses.');

INSERT INTO quests (id, name, description) VALUES ('81020ed8-9474-4eba-8256-86af495c269f', 'Pressure Drift Mapping',
'Years of tectonic pressure have nudged your relay station slightly off its original position.
The station logs every micro-adjustment it makes to remain anchored.

Individually, the movements are small.
Together, they trace a slow, wandering path across the ocean floor.

Input
Each line contains:
- A direction (N, S, E, W, NE, NW, SE, SW)
- A positive integer distance

Task
- The station starts at coordinate (0, 0).
- Apply each movement in order.
- Track the station’s position over time.

Determine the maximum Manhattan distance from the origin (0, 0) that the station reaches at any point.

Manhattan distance is defined as: |x| + |y|');

INSERT INTO quests (id, name, description) VALUES ('ad31793a-1ba3-43fc-b738-225311b7eca6', 'Signal Echo Compression',
'Signals traveling through saltwater often develop echoes.
To compensate, the relays apply a simple compression scheme, collapsing repeated values into compact representations.

A recent firmware update changed how compression is reported.
The signal itself is gone. Only the encoded form remains.

Input
A single line containing an uppercase string representing a signal.

Compression Rules
- Consecutive identical characters form a run.
- Each run contributes:
  - 1 for the character
  - number of digits in the run length

Examples:
- A -> length 1
- AAA → length 2 (A3)
- AAAAAAAAAA → length 3 (A10)

Compute the total length of the compressed signal.');

INSERT INTO quests (id, name, description) VALUES ('d45016cb-fc53-45fd-ae7f-8c44acabf3d8', 'Network Recovery Order',
'Several relays in your sector require a cold reboot.
Unfortunately, some relays depend on others being online first.

The recovery system must bring the network back up in a valid order, or risk cascading failures.

Input
Each line contains a dependency in the form: X -> Y
Meaning relay X must be rebooted before relay Y.

Determine a reboot order that satisfies all dependencies.
If multiple valid orders exist, choose the one that is lexicographically smallest when read left to right.');
