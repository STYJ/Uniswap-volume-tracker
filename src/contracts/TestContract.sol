pragma solidity ^0.7.0;

contract TestContract {
    uint storedData;

    event SetStorage(address indexed sender, uint x);

    function set(uint x) public {
        storedData = x;
        emit SetStorage(msg.sender, x);
    }

    function get() public view returns (uint) {
        return storedData;
    }
}