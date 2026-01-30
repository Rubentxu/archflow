/**
 * Properties Panel Component - Right sidebar with AWS resource properties
 */

import { Component } from './Component.js';

export class PropertiesPanel extends Component {
    constructor(app, options) {
        super(app, options);
        this.currentShape = null;
        this.awsProperties = {
            'ec2': {
                'Instance Type': 'r6g.xlarge',
                'Region': 'us-east-1 (N. Virginia)',
                'Tags': ['env:prod']
            },
            'lambda': {
                'Runtime': 'Node.js 18.x',
                'Memory': '512 MB',
                'Timeout': '3 seconds'
            },
            'eks': {
                'Version': '1.27',
                'Node Type': 't3.medium',
                'Nodes': '3'
            },
            'lightsail': {
                'Plan': 'Nano',
                'OS': 'Ubuntu 22.04',
                'IP': 'Static'
            },
            'rds': {
                'Engine': 'PostgreSQL 15',
                'Storage': '100 GB',
                'Multi-AZ': 'Yes'
            },
            'dynamodb': {
                'Capacity': 'On-demand',
                'Read Capacity': 'Unlimited',
                'Write Capacity': 'Unlimited'
            },
            'redshift': {
                'Node Type': 'dc2.large',
                'Nodes': '2',
                'Storage': '500 GB'
            },
            's3': {
                'Bucket': 'my-app-bucket',
                'Versioning': 'Enabled',
                'Encryption': 'SSE-S3'
            },
            'ebs': {
                'Size': '100 GB',
                'Type': 'gp3',
                'IOPS': '3000'
            },
            'efs': {
                'Size': '500 GB',
                'Throughput': '500 MB/s',
                'Bursting': 'Enabled'
            },
            'alb': {
                'Type': 'Application',
                'Scheme': 'Internet-facing',
                'Listeners': '80, 443'
            },
            'nlb': {
                'Type': 'Network',
                'Scheme': 'Internal',
                'Listeners': '80, 443'
            },
            'vpc': {
                'CIDR': '10.0.0.0/16',
                'Subnets': '6',
                'NAT Gateway': 'Yes'
            },
            'route53': {
                'Hosted Zones': '5',
                'Records': '42',
                'Health Checks': '8'
            }
        };
    }

    mount() {
        this.bindEvents();
        this.updateFromSelection();
        return this;
    }

    bindEvents() {
        // Section toggles
        this.element?.querySelectorAll('.section-header').forEach(header => {
            header.addEventListener('click', () => {
                this.toggleSection(header);
            });
        });

        // Transform inputs
        ['prop-x', 'prop-y', 'prop-width', 'prop-height', 'prop-rotation'].forEach(id => {
            const input = this.element?.querySelector(`#${id}`);
            input?.addEventListener('change', (e) => {
                this.handleTransformChange(id, e.target.value);
            });
        });

        // AWS Properties
        this.element?.addEventListener('change', (e) => {
            if (e.target.matches('.aws-property-select, .aws-property-input')) {
                this.updateAwsProperties();
            }
        });

        // Motion controls
        const throughputSlider = document.getElementById('throughput-slider');
        throughputSlider?.addEventListener('input', (e) => {
            const value = e.target.value;
            const label = document.getElementById('throughput-value');
            if (label) label.textContent = `${value} req/s`;
        });

        const packetSizeSlider = document.getElementById('packet-size-slider');
        packetSizeSlider?.addEventListener('input', (e) => {
            const value = e.target.value;
            const label = document.getElementById('packet-size-value');
            if (label) label.textContent = `${value}kb`;
        });

        // Alignment buttons
        this.element?.querySelectorAll('.align-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                console.log("Align clicked:", btn.title);
            });
        });
    }

    toggleSection(header) {
        const content = header.nextElementSibling;
        const toggle = header.querySelector('.section-toggle');

        if (content && content.classList.contains('collapsed')) {
            content.classList.remove('collapsed');
            toggle?.classList.remove('collapsed');
        } else {
            content.classList.add('collapsed');
            toggle?.classList.add('collapsed');
        }
    }

    handleTransformChange(id, value) {
        const selection = this.app.getEditor()?.get_selection();
        if (!selection?.shapeIds?.length) return;

        const shapeId = selection.shapeIds[0];
        console.log(`Update shape ${shapeId} ${id}: ${value}`);
    }

    updateFromSelection() {
        const selection = this.app.getEditor()?.get_selection();
        const count = selection?.shapeIds?.length || 0;

        if (count === 0) {
            this.disableInputs();
            this.currentShape = null;
            this.hideAwsProperties();
        } else if (count === 1) {
            this.enableInputs();
            this.loadShapeProperties(selection.shapeIds[0]);
        } else {
            this.enableInputs();
            this.showMultiSelection();
        }
    }

    loadShapeProperties(shapeId) {
        const shape = this.app.getEditor()?.get_shape(shapeId);
        if (!shape) return;

        this.currentShape = shape;

        // Update transform inputs
        this.setInputValue('prop-x', Math.round(shape.x));
        this.setInputValue('prop-y', Math.round(shape.y));
        this.setInputValue('prop-width', Math.round(shape.width));
        this.setInputValue('prop-height', Math.round(shape.height));
        this.setInputValue('prop-rotation', Math.round(shape.rotation));

        // Check if it's an AWS resource
        const isAwsResource = shape.properties?.type === 'aws';
        if (isAwsResource && shape.properties?.resourceId) {
            this.showAwsProperties(shape.properties.resourceId);
        } else {
            this.hideAwsProperties();
        }
    }

    showAwsProperties(resourceId) {
        const properties = this.awsProperties[resourceId];
        if (!properties) return;

        // Show AWS properties section
        const awsSection = this.element?.querySelector('#aws-properties');
        if (awsSection) {
            awsSection.classList.remove('hidden');

            // Update AWS properties
            const resourceName = this.element?.querySelector('#aws-resource-name');
            const instanceType = this.element?.querySelector('#aws-instance-type');
            const region = this.element?.querySelector('#aws-region');
            const tags = this.element?.querySelector('#aws-tags');

            if (resourceName) resourceName.textContent = properties['Instance Type'] || properties['Runtime'] || properties['Version'] || resourceId;
            if (instanceType) instanceType.textContent = properties['Instance Type'] || 'N/A';
            if (region) region.textContent = properties['Region'] || 'N/A';
            if (tags) {
                const tagsHtml = properties['Tags'] ? properties['Tags'].map(tag =>
                    `<span class="tag">${tag}</span>`
                ).join('') : '<span class="tag">No tags</span>';
                tags.innerHTML = tagsHtml;
            }
        }
    }

    hideAwsProperties() {
        const awsSection = this.element?.querySelector('#aws-properties');
        if (awsSection) {
            awsSection.classList.add('hidden');
        }
    }

    updateAwsProperties() {
        // Handle AWS property changes
        const resourceId = this.currentShape?.properties?.resourceId;
        if (!resourceId) return;

        const instanceTypeSelect = document.getElementById('aws-instance-type-select');
        const regionSelect = document.getElementById('aws-region-select');

        if (instanceTypeSelect && regionSelect) {
            // Update shape properties
            const shape = this.currentShape;
            shape.properties.instanceType = instanceTypeSelect.value;
            shape.properties.region = regionSelect.value;

            console.log(`Updated AWS properties for ${resourceId}:`, {
                instanceType: instanceTypeSelect.value,
                region: regionSelect.value
            });

            // Update visual representation if needed
            this.updateShapeVisuals();
        }
    }

    updateShapeVisuals() {
        const shape = this.currentShape;
        if (!shape) return;

        // Update shape appearance based on AWS properties
        const resourceId = shape.properties.resourceId;
        const instanceType = shape.properties.instanceType;
        const region = shape.properties.region;

        // Update shape text/content
        const resourceName = this.element?.querySelector('#aws-resource-name');
        if (resourceName) {
            const baseName = this.awsProperties[resourceId]?.['Instance Type'] ||
                this.awsProperties[resourceId]?.['Runtime'] ||
                this.awsProperties[resourceId]?.['Version'] ||
                resourceId;
            resourceName.textContent = `${baseName} (${instanceType})`;
        }

        // Update shape color based on region or instance type
        const colorMap = {
            'us-east-1': '#FF9900',
            'us-west-1': '#0066CC',
            'eu-west-1': '#00CC66',
            'ap-southeast-1': '#CC66CC',
            'r6g.xlarge': '#FF9900',
            'r6g.2xlarge': '#0066CC',
            'r6g.4xlarge': '#00CC66',
            'r6g.8xlarge': '#CC66CC'
        };

        const color = colorMap[region] || colorMap[instanceType] || shape.fill;
        if (color !== shape.fill) {
            shape.fill = color;
            this.app.getEditor()?.update_shape(shape.id, { fill: color });
        }
    }

    disableInputs() {
        this.element?.querySelectorAll('input, button, select').forEach(el => {
            el.disabled = true;
            el.classList.add('disabled');
        });
    }

    enableInputs() {
        this.element?.querySelectorAll('input, button, select').forEach(el => {
            el.disabled = false;
            el.classList.remove('disabled');
        });
    }

    showMultiSelection() {
        console.log('Multi-selection - properties limited');
        this.setInputValue('prop-x', '-');
        this.setInputValue('prop-y', '-');
        this.setInputValue('prop-width', '-');
        this.setInputValue('prop-height', '-');
    }

    setInputValue(id, value) {
        const input = this.element?.querySelector(`#${id}`);
        if (input) {
            input.value = value;
        }
    }
}
