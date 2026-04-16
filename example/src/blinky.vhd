library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity blinky is
    port (
        clk : in  std_logic;
        led : out std_logic_vector(3 downto 0)
    );
end entity blinky;

architecture rtl of blinky is
    signal counter : unsigned(25 downto 0) := (others => '0');
begin
    process (clk)
    begin
        if rising_edge(clk) then
            counter <= counter + 1;
        end if;
    end process;

    led(0) <= counter(25);
    led(1) <= counter(24);
    led(2) <= counter(23);
    led(3) <= counter(22);
end architecture rtl;
