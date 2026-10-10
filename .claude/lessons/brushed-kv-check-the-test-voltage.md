# Brushed motor tables: confirm the test voltage before deriving kv

A brushed motor's kv and R come from free speed and stall current at *one*
test voltage, and secondary tables don't always test at the rated voltage.
The FRC 2012 motor table (penfieldrobotics wiki) lists the BaneBots
RS-775-18 at **12 V** (13000 rpm, 86.7 A), while the vendor page lists it
at 18 V (19500 rpm, 130 A). Reading the 12 V row as 18 V gave kv 722 and
R 0.208 Ω instead of the real 1083 and 0.138 (2026-10-10).

Next time: derive kv and R from the vendor page, and use any second table
only as a cross-check. Two sources should agree on kv = rpm/V and
R = V/I_stall. If they don't, one of them was read at the wrong voltage.
