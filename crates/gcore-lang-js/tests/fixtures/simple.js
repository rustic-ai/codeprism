// Simple JavaScript test file

function greet(name) {
    console.log(`Hello, ${name}!`);
}

const message = "World";
greet(message);

class Person {
    constructor(name) {
        this.name = name;
    }
    
    sayHello() {
        greet(this.name);
    }
}

const person = new Person("Alice");
person.sayHello(); 