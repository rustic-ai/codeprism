// TypeScript test file with type annotations

interface User {
    id: number;
    name: string;
    email: string;
}

class UserService {
    private users: Map<number, User> = new Map();
    
    constructor(private apiUrl: string) {}
    
    async getUser(id: number): Promise<User | null> {
        const cached = this.users.get(id);
        if (cached) {
            return cached;
        }
        
        const response = await fetch(`${this.apiUrl}/users/${id}`);
        if (!response.ok) {
            return null;
        }
        
        const user = await response.json() as User;
        this.users.set(id, user);
        return user;
    }
    
    async createUser(data: Omit<User, 'id'>): Promise<User> {
        const response = await fetch(this.apiUrl + '/users', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(data)
        });
        
        return response.json();
    }
}

export { UserService, User }; 