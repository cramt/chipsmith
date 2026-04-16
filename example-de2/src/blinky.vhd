library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity blinky is
    port (
        CLOCK_50 : in  std_logic;
        LEDR     : out std_logic_vector(3 downto 0)
    );
end entity blinky;

architecture rtl of blinky is
    signal counter : unsigned(25 downto 0) := (others => '0');
begin
    process (CLOCK_50)
    begin
        if rising_edge(CLOCK_50) then
            counter <= counter + 1;
        end if;
    end process;

    LEDR(0) <= counter(25);
    LEDR(1) <= counter(24);
    LEDR(2) <= counter(23);
    LEDR(3) <= counter(22);
end architecture rtl;
