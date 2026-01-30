/**
 * Component Library Panel - Left sidebar with AWS resources and draggable components
 */

import { Component } from './Component.js';

export class ComponentLibrary extends Component {
    constructor(app, options) {
        super(app, options);
        this.awsResources = {
            compute: [
                { id: 'ec2', name: 'EC2', icon: 'server', color: '#FF9900' },
                { id: 'lambda', name: 'Lambda', icon: 'lambda', color: '#FF9900' },
                { id: 'eks', name: 'EKS', icon: 'kubernetes', color: '#FF9900' },
                { id: 'lightsail', name: 'Lightsail', icon: 'lightbulb', color: '#FF9900' }
            ],
            database: [
                { id: 'rds', name: 'RDS', icon: 'database', color: '#0066CC' },
                { id: 'dynamodb', name: 'DynamoDB', icon: 'table', color: '#0066CC' },
                { id: 'redshift', name: 'Redshift', icon: 'cube', color: '#0066CC' }
            ],
            storage: [
                { id: 's3', name: 'S3', icon: 'hard-drive', color: '#00CC66' },
                { id: 'ebs', name: 'EBS', icon: 'hard-drive', color: '#00CC66' },
                { id: 'efs', name: 'EFS', icon: 'hard-drive', color: '#00CC66' }
            ],
            networking: [
                { id: 'alb', name: 'ALB', icon: 'network', color: '#0099CC' },
                { id: 'nlb', name: 'NLB', icon: 'network', color: '#0099CC' },
                { id: 'vpc', name: 'VPC', icon: 'network', color: '#0099CC' },
                { id: 'route53', name: 'Route 53', icon: 'globe', color: '#0099CC' }
            ]
        };
        this.collapsedCategories = new Set();
    }

    mount() {
        this.renderCategories();
        this.setupSearch();
        this.bindEvents();
        return this;
    }

    bindEvents() {
        // Category toggle
        const categoriesContainer = document.getElementById('library-categories');
        categoriesContainer?.addEventListener('click', (e) => {
            const header = e.target.closest('.category-header');
            if (header) {
                const category = header.closest('.category');
                category.classList.toggle('open');
                const categoryId = category.dataset.category;
                if (category.classList.contains('open')) {
                    this.collapsedCategories.delete(categoryId);
                } else {
                    this.collapsedCategories.add(categoryId);
                }
            }
        });

        // Drag and drop
        categoriesContainer?.addEventListener('dragstart', (e) => {
            const item = e.target.closest('.library-item');
            if (item) {
                this.handleDragStart(e, item);
            }
        });

        categoriesContainer?.addEventListener('dragend', (e) => {
            const item = e.target.closest('.library-item');
            if (item) {
                item.classList.remove('dragging');
            }
        });
    }

    renderCategories() {
        const categoriesContainer = document.getElementById('library-categories');
        if (!categoriesContainer) return;

        categoriesContainer.innerHTML = '';

        // AWS Resources Categories
        Object.entries(this.awsResources).forEach(([categoryId, items]) => {
            const categoryEl = document.createElement('div');
            categoryEl.className = 'category';
            categoryEl.dataset.category = categoryId;

            if (!this.collapsedCategories.has(categoryId)) {
                categoryEl.classList.add('open');
            }

            categoryEl.innerHTML = `
                <div class="category-header">
                    <i class="ph ph-caret-right"></i>
                    <span class="category-title">${this.getCategoryName(categoryId)}</span>
                    <span class="category-count">${items.length}</span>
                </div>
                <div class="category-items"></div>
            `;

            const itemsContainer = categoryEl.querySelector('.category-items');
            items.forEach(item => {
                const itemEl = this.createComponentElement(categoryId, item);
                itemsContainer.appendChild(itemEl);
            });

            categoriesContainer.appendChild(categoryEl);
        });
    }

    getCategoryName(categoryId) {
        const names = {
            compute: 'Compute',
            database: 'Database',
            storage: 'Storage',
            networking: 'Networking'
        };
        return names[categoryId] || 'General';
    }

    createComponentElement(categoryId, item) {
        const itemEl = document.createElement('div');
        itemEl.className = 'library-item';
        itemEl.draggable = true;
        itemEl.dataset.type = item.id;
        itemEl.dataset.categoryId = categoryId;
        itemEl.dataset.itemId = item.id;

        const iconClass = `aws-${categoryId}`;
        itemEl.innerHTML = `
            <i class="ph-fill ph-${item.icon} ${iconClass}"></i>
            <span>${item.name}</span>
        `;

        return itemEl;
    }

    setupSearch() {
        const searchInput = document.getElementById('library-search-input');
        searchInput?.addEventListener('input', (e) => {
            const query = e.target.value.toLowerCase();
            this.filterItems(query);
        });
    }

    filterItems(query) {
        const categoriesContainer = document.getElementById('library-categories');
        categoriesContainer?.querySelectorAll('.library-item').forEach(item => {
            const label = item.querySelector('span')?.textContent.toLowerCase() || '';
            const matches = label.includes(query);
            item.style.display = matches ? '' : 'none';
        });

        // Also filter categories
        categoriesContainer?.querySelectorAll('.category').forEach(category => {
            const visibleItems = category.querySelectorAll('.library-item:not([style*="display: none"])');
            const hasVisible = visibleItems.length > 0;
            category.style.display = hasVisible ? '' : 'none';
        });
    }

    handleDragStart(e, item) {
        item.classList.add('dragging');

        const data = {
            categoryId: item.dataset.categoryId,
            itemId: item.dataset.itemId,
            type: 'application/archflow-component'
        };

        e.dataTransfer.setData('application/archflow-component', JSON.stringify(data));

        // Custom drag image if needed
    }

    handleKeyboardDrop(item) {
        const canvas = this.app.getComponent('canvas');
        const rect = canvas?.element?.getBoundingClientRect();

        if (rect) {
            const x = rect.width / 2;
            const y = rect.height / 2;

            const categoryId = item.dataset.categoryId;
            const itemId = item.dataset.itemId;

            this.createShapeFromLibrary(categoryId, itemId, x, y);
        }
    }

    createShapeFromLibrary(categoryId, itemId, x, y) {
        const editor = this.app.getEditor();
        if (editor) {
            const resource = this.awsResources[categoryId]?.find(r => r.id === itemId);
            if (resource) {
                const shapeId = editor.create_shape(
                    'aws-resource',
                    x - 60,
                    y - 40,
                    120,
                    80,
                    resource.color,
                    {
                        type: 'aws',
                        resourceId: itemId,
                        resourceName: resource.name,
                        category: categoryId
                    }
                );

                if (shapeId) {
                    console.log(`Created AWS resource ${resource.name}: ${shapeId}`);
                }
            }
        }
    }
}
