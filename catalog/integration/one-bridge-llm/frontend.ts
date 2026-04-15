import type { NodeTemplate, ValidationContext, ValidationError } from '$lib/types';
import { BrainCircuit } from '@lucide/svelte';
import { isInputConnected } from '$lib/validation';

export const OneBridgeLlmNode: NodeTemplate = {
	type: 'OneBridgeLlm',
	label: 'ONE Bridge LLM',
	description: 'Call Claude through the ONE cc-bridge infrastructure',
	icon: BrainCircuit,
	color: '#4a9eff',
	category: 'AI',
	tags: ['ai', 'claude', 'bridge', 'one', 'llm', 'integration'],
	fields: [
		{ key: 'bridge_url', label: 'Bridge URL', type: 'text', placeholder: 'http://localhost:8070', description: 'URL of the cc-bridge instance' },
		{ key: 'max_tokens', label: 'Max tokens', type: 'number', placeholder: '4096' },
		{ key: 'temperature', label: 'Temperature', type: 'number', placeholder: '(default)', description: '0.0-1.0. Higher = more creative.' },
	],
	defaultInputs: [
		{ name: 'prompt', portType: 'String', required: true, description: 'The prompt to send to the LLM' },
		{ name: 'system_prompt', portType: 'String', required: false, description: 'Optional system prompt' },
		{ name: 'model', portType: 'String', required: false, description: 'Model override (default: bridge config)' },
	],
	defaultOutputs: [
		{ name: 'response', portType: 'String', required: false, description: 'LLM response text' },
		{ name: 'model', portType: 'String', required: false, description: 'Model that was used' },
		{ name: 'input_tokens', portType: 'Number', required: false, description: 'Input token count' },
		{ name: 'output_tokens', portType: 'Number', required: false, description: 'Output token count' },
	],
	features: {
		canAddInputPorts: false,
		canAddOutputPorts: false,
	},
	validate: (context: ValidationContext): ValidationError[] => {
		const errors: ValidationError[] = [];

		if (!isInputConnected('prompt', context)) {
			errors.push({ port: 'prompt', message: 'Prompt input is required', level: 'structural' });
		}

		return errors;
	},
};
