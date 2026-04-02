---
name: static-site
description: Simple HTML/CSS/JS static website
language: html
docker_image: nginx:alpine
variables:
  - name: project_name
    description: Site name
    required: true
  - name: title
    description: Page title
    default: "{{ project_name }}"
tags: [frontend, static, html]
---

# {{ project_name }}

A simple, fast static website with HTML, CSS, and vanilla JavaScript.

## Project Structure

```
{{ project_name }}/
├── index.html
├── about.html
├── contact.html
├── styles.css
├── main.js
├── assets/
│   ├── images/
│   │   └── logo.png
│   └── fonts/
├── nginx.conf
├── Dockerfile
├── docker-compose.yml
└── README.md
```

## Files

### index.html
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="Welcome to {{ project_name }}">
    <title>{{ title }} - Home</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <header>
        <nav class="navbar">
            <div class="container">
                <div class="logo">{{ project_name }}</div>
                <ul class="nav-links">
                    <li><a href="index.html" class="active">Home</a></li>
                    <li><a href="about.html">About</a></li>
                    <li><a href="contact.html">Contact</a></li>
                </ul>
            </div>
        </nav>
    </header>

    <main>
        <section class="hero">
            <div class="container">
                <h1>Welcome to {{ project_name }}</h1>
                <p class="subtitle">A modern, fast static website</p>
                <button class="cta-button" onclick="scrollToSection('features')">Learn More</button>
            </div>
        </section>

        <section id="features" class="features">
            <div class="container">
                <h2>Features</h2>
                <div class="feature-grid">
                    <div class="feature-card">
                        <h3>⚡ Fast</h3>
                        <p>Lightning-fast load times with static HTML</p>
                    </div>
                    <div class="feature-card">
                        <h3>🔒 Secure</h3>
                        <p>No server-side vulnerabilities to worry about</p>
                    </div>
                    <div class="feature-card">
                        <h3>📱 Responsive</h3>
                        <p>Works perfectly on all devices and screen sizes</p>
                    </div>
                    <div class="feature-card">
                        <h3>🎨 Beautiful</h3>
                        <p>Modern design with smooth animations</p>
                    </div>
                </div>
            </div>
        </section>

        <section class="cta-section">
            <div class="container">
                <h2>Ready to get started?</h2>
                <p>Join us today and experience the difference</p>
                <a href="contact.html" class="cta-button">Contact Us</a>
            </div>
        </section>
    </main>

    <footer>
        <div class="container">
            <p>&copy; 2024 {{ project_name }}. All rights reserved.</p>
            <div class="social-links">
                <a href="#" target="_blank">Twitter</a>
                <a href="#" target="_blank">GitHub</a>
                <a href="#" target="_blank">LinkedIn</a>
            </div>
        </div>
    </footer>

    <script src="main.js"></script>
</body>
</html>
```

### about.html
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="About {{ project_name }}">
    <title>{{ title }} - About</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <header>
        <nav class="navbar">
            <div class="container">
                <div class="logo">{{ project_name }}</div>
                <ul class="nav-links">
                    <li><a href="index.html">Home</a></li>
                    <li><a href="about.html" class="active">About</a></li>
                    <li><a href="contact.html">Contact</a></li>
                </ul>
            </div>
        </nav>
    </header>

    <main>
        <section class="page-header">
            <div class="container">
                <h1>About {{ project_name }}</h1>
            </div>
        </section>

        <section class="content">
            <div class="container">
                <div class="about-content">
                    <h2>Our Story</h2>
                    <p>
                        {{ project_name }} was created with a mission to provide a simple, 
                        fast, and reliable solution for modern web projects. We believe in 
                        the power of static sites and the importance of performance.
                    </p>

                    <h2>Our Mission</h2>
                    <p>
                        To deliver exceptional web experiences through clean code, 
                        thoughtful design, and a commitment to excellence.
                    </p>

                    <h2>Why Choose Us?</h2>
                    <ul class="benefits-list">
                        <li>✓ Industry-leading performance</li>
                        <li>✓ Secure and reliable infrastructure</li>
                        <li>✓ 24/7 support and maintenance</li>
                        <li>✓ Scalable solutions for any size project</li>
                        <li>✓ Transparent pricing with no hidden fees</li>
                    </ul>
                </div>
            </div>
        </section>
    </main>

    <footer>
        <div class="container">
            <p>&copy; 2024 {{ project_name }}. All rights reserved.</p>
            <div class="social-links">
                <a href="#" target="_blank">Twitter</a>
                <a href="#" target="_blank">GitHub</a>
                <a href="#" target="_blank">LinkedIn</a>
            </div>
        </div>
    </footer>

    <script src="main.js"></script>
</body>
</html>
```

### contact.html
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="Contact {{ project_name }}">
    <title>{{ title }} - Contact</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <header>
        <nav class="navbar">
            <div class="container">
                <div class="logo">{{ project_name }}</div>
                <ul class="nav-links">
                    <li><a href="index.html">Home</a></li>
                    <li><a href="about.html">About</a></li>
                    <li><a href="contact.html" class="active">Contact</a></li>
                </ul>
            </div>
        </nav>
    </header>

    <main>
        <section class="page-header">
            <div class="container">
                <h1>Contact Us</h1>
            </div>
        </section>

        <section class="content">
            <div class="container">
                <div class="contact-container">
                    <form class="contact-form" id="contactForm">
                        <div class="form-group">
                            <label for="name">Name</label>
                            <input type="text" id="name" name="name" required>
                        </div>

                        <div class="form-group">
                            <label for="email">Email</label>
                            <input type="email" id="email" name="email" required>
                        </div>

                        <div class="form-group">
                            <label for="subject">Subject</label>
                            <input type="text" id="subject" name="subject" required>
                        </div>

                        <div class="form-group">
                            <label for="message">Message</label>
                            <textarea id="message" name="message" rows="6" required></textarea>
                        </div>

                        <button type="submit" class="cta-button">Send Message</button>
                    </form>

                    <div class="contact-info">
                        <h3>Get in Touch</h3>
                        <p>
                            Have a question or want to work together? 
                            We'd love to hear from you!
                        </p>
                        <div class="info-item">
                            <strong>Email:</strong>
                            <a href="mailto:hello@{{ project_name }}.com">hello@{{ project_name }}.com</a>
                        </div>
                        <div class="info-item">
                            <strong>Phone:</strong>
                            <a href="tel:+1234567890">+1 (234) 567-890</a>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    </main>

    <footer>
        <div class="container">
            <p>&copy; 2024 {{ project_name }}. All rights reserved.</p>
            <div class="social-links">
                <a href="#" target="_blank">Twitter</a>
                <a href="#" target="_blank">GitHub</a>
                <a href="#" target="_blank">LinkedIn</a>
            </div>
        </div>
    </footer>

    <script src="main.js"></script>
</body>
</html>
```

### styles.css
```css
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

:root {
    --primary-color: #0066cc;
    --secondary-color: #f0f0f0;
    --text-color: #333;
    --border-color: #ddd;
    --transition: all 0.3s ease;
}

html {
    scroll-behavior: smooth;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    line-height: 1.6;
    color: var(--text-color);
    background-color: #fff;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 20px;
}

/* Header & Navigation */
header {
    background-color: #fff;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    position: sticky;
    top: 0;
    z-index: 100;
}

.navbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 0;
}

.navbar .container {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
}

.logo {
    font-size: 1.5rem;
    font-weight: bold;
    color: var(--primary-color);
}

.nav-links {
    display: flex;
    list-style: none;
    gap: 2rem;
}

.nav-links a {
    text-decoration: none;
    color: var(--text-color);
    transition: var(--transition);
    font-weight: 500;
}

.nav-links a:hover,
.nav-links a.active {
    color: var(--primary-color);
}

/* Hero Section */
.hero {
    background: linear-gradient(135deg, var(--primary-color) 0%, #0052a3 100%);
    color: white;
    padding: 6rem 0;
    text-align: center;
}

.hero h1 {
    font-size: 3rem;
    margin-bottom: 1rem;
}

.hero .subtitle {
    font-size: 1.25rem;
    margin-bottom: 2rem;
    opacity: 0.9;
}

/* Features Section */
.features {
    padding: 4rem 0;
    background-color: var(--secondary-color);
}

.features h2 {
    text-align: center;
    font-size: 2rem;
    margin-bottom: 3rem;
}

.feature-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 2rem;
}

.feature-card {
    background: white;
    padding: 2rem;
    border-radius: 8px;
    text-align: center;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    transition: var(--transition);
}

.feature-card:hover {
    transform: translateY(-5px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.feature-card h3 {
    font-size: 1.5rem;
    margin-bottom: 1rem;
}

/* CTA Section */
.cta-section {
    background: linear-gradient(135deg, var(--primary-color) 0%, #0052a3 100%);
    color: white;
    padding: 4rem 0;
    text-align: center;
}

.cta-section h2 {
    font-size: 2rem;
    margin-bottom: 1rem;
}

.cta-section p {
    font-size: 1.1rem;
    margin-bottom: 2rem;
}

/* Buttons */
.cta-button {
    background-color: var(--primary-color);
    color: white;
    border: none;
    padding: 0.75rem 2rem;
    font-size: 1rem;
    border-radius: 4px;
    cursor: pointer;
    transition: var(--transition);
    text-decoration: none;
    display: inline-block;
}

.cta-button:hover {
    background-color: #0052a3;
    transform: scale(1.05);
}

/* Page Header */
.page-header {
    background: linear-gradient(135deg, var(--primary-color) 0%, #0052a3 100%);
    color: white;
    padding: 3rem 0;
    text-align: center;
}

.page-header h1 {
    font-size: 2.5rem;
}

/* Content Section */
.content {
    padding: 4rem 0;
}

.about-content h2 {
    font-size: 1.5rem;
    margin-top: 2rem;
    margin-bottom: 1rem;
    color: var(--primary-color);
}

.about-content p {
    margin-bottom: 1rem;
    line-height: 1.8;
}

.benefits-list {
    list-style: none;
    margin: 2rem 0;
}

.benefits-list li {
    padding: 0.5rem 0;
    font-size: 1.1rem;
}

/* Contact Form */
.contact-container {
    display: grid;
    grid-template-columns: 2fr 1fr;
    gap: 3rem;
}

.contact-form {
    background: var(--secondary-color);
    padding: 2rem;
    border-radius: 8px;
}

.form-group {
    margin-bottom: 1.5rem;
}

.form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
}

.form-group input,
.form-group textarea {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-family: inherit;
    font-size: 1rem;
}

.form-group input:focus,
.form-group textarea:focus {
    outline: none;
    border-color: var(--primary-color);
    box-shadow: 0 0 0 3px rgba(0, 102, 204, 0.1);
}

.contact-info {
    background: var(--secondary-color);
    padding: 2rem;
    border-radius: 8px;
}

.contact-info h3 {
    font-size: 1.5rem;
    margin-bottom: 1rem;
    color: var(--primary-color);
}

.info-item {
    margin: 1.5rem 0;
}

.info-item a {
    color: var(--primary-color);
    text-decoration: none;
}

.info-item a:hover {
    text-decoration: underline;
}

/* Footer */
footer {
    background-color: #333;
    color: white;
    padding: 2rem 0;
    text-align: center;
    margin-top: 4rem;
}

.social-links {
    margin-top: 1rem;
}

.social-links a {
    color: white;
    text-decoration: none;
    margin: 0 1rem;
    transition: var(--transition);
}

.social-links a:hover {
    color: var(--primary-color);
}

/* Responsive */
@media (max-width: 768px) {
    .nav-links {
        gap: 1rem;
    }

    .hero h1 {
        font-size: 2rem;
    }

    .feature-grid {
        grid-template-columns: 1fr;
    }

    .contact-container {
        grid-template-columns: 1fr;
    }

    .page-header h1 {
        font-size: 1.75rem;
    }
}
```

### main.js
```javascript
// Smooth scroll to section
function scrollToSection(sectionId) {
    const section = document.getElementById(sectionId);
    if (section) {
        section.scrollIntoView({ behavior: 'smooth' });
    }
}

// Handle contact form submission
document.addEventListener('DOMContentLoaded', function() {
    const contactForm = document.getElementById('contactForm');
    
    if (contactForm) {
        contactForm.addEventListener('submit', function(e) {
            e.preventDefault();
            
            const formData = new FormData(contactForm);
            const data = Object.fromEntries(formData);
            
            console.log('Form submitted:', data);
            
            // Show success message
            alert('Thank you for your message! We will get back to you soon.');
            contactForm.reset();
        });
    }
    
    // Update active nav link based on current page
    const currentPage = window.location.pathname.split('/').pop() || 'index.html';
    document.querySelectorAll('.nav-links a').forEach(link => {
        const href = link.getAttribute('href');
        if (href === currentPage) {
            link.classList.add('active');
        } else {
            link.classList.remove('active');
        }
    });
});

// Add scroll animation
window.addEventListener('scroll', function() {
    const navbar = document.querySelector('header');
    if (window.scrollY > 50) {
        navbar.style.boxShadow = '0 4px 8px rgba(0, 0, 0, 0.15)';
    } else {
        navbar.style.boxShadow = '0 2px 4px rgba(0, 0, 0, 0.1)';
    }
});
```

### nginx.conf
```nginx
server {
    listen 80;
    server_name _;

    root /usr/share/nginx/html;
    index index.html;

    # Gzip compression
    gzip on;
    gzip_types text/plain text/css text/javascript application/json;
    gzip_min_length 1000;

    # Cache static assets
    location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg|woff|woff2|ttf|eot)$ {
        expires 1y;
        add_header Cache-Control "public, immutable";
    }

    # HTML files - no cache
    location ~* \.html$ {
        expires -1;
        add_header Cache-Control "no-cache, no-store, must-revalidate";
    }

    # SPA routing - serve index.html for all routes
    location / {
        try_files $uri $uri/ /index.html;
    }

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "no-referrer-when-downgrade" always;
}
```

### Dockerfile
```dockerfile
FROM nginx:alpine

COPY . /usr/share/nginx/html/
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
```

### docker-compose.yml
```yaml
version: '3.8'

services:
  web:
    build: .
    ports:
      - "80:80"
    volumes:
      - .:/usr/share/nginx/html
    environment:
      - NGINX_HOST={{ project_name }}.local
      - NGINX_PORT=80
```

## Getting Started

1. Open `index.html` in your browser or serve with a local server:
   ```bash
   python -m http.server 8000
   ```

2. Or use Docker:
   ```bash
   docker-compose up
   ```

3. Visit `http://localhost` or `http://localhost:8000`

## Deployment

### Deploy to Netlify
```bash
netlify deploy --prod --dir .
```

### Deploy to Vercel
```bash
vercel --prod
```

### Deploy to GitHub Pages
Push to `gh-pages` branch or configure in repository settings.

## Features

- ✅ Fully responsive design
- ✅ Fast loading times
- ✅ SEO optimized
- ✅ Accessible HTML
- ✅ Modern CSS with animations
- ✅ Vanilla JavaScript (no dependencies)
- ✅ Docker ready
- ✅ Nginx configuration included
