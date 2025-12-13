// Clear existing data
MATCH (n) DETACH DELETE n;

// ==================== NODES ====================

// Languages
CREATE (rust:Language {id: 1, name: 'Rust', type: 'Compiled', first_release: 2010, paradigm: 'Multi-paradigm'})
CREATE (go:Language {id: 2, name: 'Go', type: 'Compiled', first_release: 2009, paradigm: 'Imperative'})
CREATE (gleam:Language {id: 3, name: 'Gleam', type: 'Compiled to JS/Erlang', first_release: 2019, paradigm: 'Functional'})
CREATE (erlang:Language {id: 4, name: 'Erlang', type: 'Interpreted', first_release: 1986, paradigm: 'Functional'})
CREATE (python:Language {id: 5, name: 'Python', type: 'Interpreted', first_release: 1991, paradigm: 'Multi-paradigm'})
CREATE (typescript:Language {id: 6, name: 'TypeScript', type: 'Compiled to JS', first_release: 2012, paradigm: 'Multi-paradigm'})
CREATE (c:Language {id: 7, name: 'C', type: 'Compiled', first_release: 1972, paradigm: 'Imperative'})

// Features
CREATE (f1:Feature {id: 1, name: 'Strong Type System', category: 'Type System'})
CREATE (f2:Feature {id: 2, name: 'Garbage Collection', category: 'Memory Management'})
CREATE (f3:Feature {id: 3, name: 'Concurrency Primitives', category: 'Concurrency'})
CREATE (f4:Feature {id: 4, name: 'Pattern Matching', category: 'Language Features'})
CREATE (f5:Feature {id: 5, name: 'First-Class Functions', category: 'Functional'})
CREATE (f6:Feature {id: 6, name: 'Immutability by Default', category: 'Functional'})
CREATE (f7:Feature {id: 7, name: 'Memory Safety', category: 'Memory Management'})
CREATE (f8:Feature {id: 8, name: 'Goroutines', category: 'Concurrency'})
CREATE (f9:Feature {id: 9, name: 'Zero-Cost Abstractions', category: 'Performance'})
CREATE (f10:Feature {id: 10, name: 'Hot Code Reload', category: 'Runtime Features'})
CREATE (f11:Feature {id: 11, name: 'Ownership System', category: 'Memory Management'})

// Applications
CREATE (a1:Application {id: 1, name: 'Systems Programming', domain: 'Low-level', maturity: 'Production'})
CREATE (a2:Application {id: 2, name: 'Web Servers', domain: 'Web', maturity: 'Production'})
CREATE (a3:Application {id: 3, name: 'Distributed Systems', domain: 'Infrastructure', maturity: 'Production'})
CREATE (a4:Application {id: 4, name: 'Real-time Applications', domain: 'Telecom', maturity: 'Production'})
CREATE (a5:Application {id: 5, name: 'Data Science', domain: 'Data Analytics', maturity: 'Production'})
CREATE (a6:Application {id: 6, name: 'Frontend Development', domain: 'Web', maturity: 'Production'})
CREATE (a7:Application {id: 7, name: 'CLI Tools', domain: 'Tooling', maturity: 'Production'})
CREATE (a8:Application {id: 8, name: 'Embedded Systems', domain: 'Hardware', maturity: 'Production'})
CREATE (a9:Application {id: 9, name: 'Blockchain', domain: 'Crypto', maturity: 'Emerging'})
CREATE (a10:Application {id: 10, name: 'Game Development', domain: 'Gaming', maturity: 'Production'})

// ==================== RELATIONSHIPS ====================

// Features
CREATE (rust)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f1)
CREATE (rust)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f7)
CREATE (rust)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f11)
CREATE (rust)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f9)
CREATE (rust)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f3)

CREATE (go)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f2)
CREATE (go)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f1)
CREATE (go)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f8)
CREATE (go)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f3)

CREATE (gleam)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f1)
CREATE (gleam)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f4)
CREATE (gleam)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f5)
CREATE (gleam)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f6)

CREATE (erlang)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f3)
CREATE (erlang)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f4)
CREATE (erlang)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f10)
CREATE (erlang)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f5)

CREATE (python)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f2)
CREATE (python)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f5)

CREATE (typescript)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f1)
CREATE (typescript)-[:HAS_FEATURE {support_level: 'Built-in'}]->(f5)

CREATE (c)-[:HAS_FEATURE {support_level: 'None'}]->(f7)

// Language Influence
CREATE (rust)-[:INFLUENCED_BY {year: 2010}]->(c)
CREATE (go)-[:INFLUENCED_BY {year: 2009}]->(c)
CREATE (gleam)-[:INFLUENCED_BY {year: 2019}]->(erlang)
CREATE (gleam)-[:INFLUENCED_BY {year: 2019}]->(typescript)
CREATE (typescript)-[:INFLUENCED_BY {year: 2012}]->(python)
CREATE (erlang)-[:INFLUENCED_BY {year: 1986}]->(c)

// Applications
CREATE (rust)-[:SUITED_FOR {popularity: 'High'}]->(a1)
CREATE (rust)-[:SUITED_FOR {popularity: 'High'}]->(a2)
CREATE (rust)-[:SUITED_FOR {popularity: 'High'}]->(a7)

CREATE (go)-[:SUITED_FOR {popularity: 'High'}]->(a2)
CREATE (go)-[:SUITED_FOR {popularity: 'High'}]->(a3)
CREATE (go)-[:SUITED_FOR {popularity: 'High'}]->(a7)

CREATE (gleam)-[:SUITED_FOR {popularity: 'Medium'}]->(a2)
CREATE (gleam)-[:SUITED_FOR {popularity: 'Medium'}]->(a4)

CREATE (erlang)-[:SUITED_FOR {popularity: 'High'}]->(a4)
CREATE (erlang)-[:SUITED_FOR {popularity: 'Medium'}]->(a3)

CREATE (python)-[:SUITED_FOR {popularity: 'High'}]->(a5)
CREATE (python)-[:SUITED_FOR {popularity: 'Medium'}]->(a2)

CREATE (typescript)-[:SUITED_FOR {popularity: 'High'}]->(a6)
CREATE (typescript)-[:SUITED_FOR {popularity: 'Medium'}]->(a2)

CREATE (c)-[:SUITED_FOR {popularity: 'High'}]->(a1)
CREATE (c)-[:SUITED_FOR {popularity: 'High'}]->(a8)
